#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod record;
mod transcribe;

use std::sync::{Arc, Mutex, mpsc};
use tauri::State;
use tokio::sync::oneshot;
use tempfile::NamedTempFile;

fn main() {
    // Failing to establish this would be a critical error
    let record_file = NamedTempFile::new().expect("failed to create temporary file target for recording");

    let (transcription_action_tx, transcription_action_rx) = mpsc::channel();
    let (transcription_tx, transcription_rx) = mpsc::channel();

    // We need to start a separate thread for the transcription process, which must hold
    // the global embedded Python interpreter lock. This interpreter can't be re-initialised,
    // so we have to hold it in a separate slave thread.
    let record_file_path = record_file.path().to_string_lossy().to_string();
    let handle = std::thread::spawn(move || {
        let action_rx = transcription_action_rx;
        let result_tx = transcription_tx;

        while action_rx.recv().is_ok() {
            let transcription_res = transcribe::transcribe(&record_file_path);
            // The receiver is maintained by the global state, so if this fails, the app has been closed
            let _ = result_tx.send(transcription_res);
        }

        // // SAFETY: The global Python interpreter cannot be invoked more than once,
        // // so we consign it to a separate thread with message-passing.
        // unsafe {
        //     pyo3::with_embedded_python_interpreter(|py| {
        //         let py_code = include_str!("transcribe.py");
        //         // Hilariously, Python won't let us call this `whisper`.
        //         // If this panics, the app will fail immediately before it can be opened,
        //         // preventing the user from recording anything that they might then lose.
        //         // (Aka. we're calling this a feature!)
        //         let whisper = PyModule::from_code(py, py_code, "transcribe.py", "transcribe")
        //             .expect("failed to instantiate whisper submodule");
        //         while action_rx.recv().is_ok() {
        //             // We've received a new instruction to transcribe the latest recording
        //             // We use a closure here to make using `?` easy
        //             let transcription_res: Result<String, String> = (|| {
        //                 let transcription = whisper
        //                     .getattr("transcribe")
        //                     .map_err(|err| err.to_string())?
        //                     .call1((&record_file_path,))
        //                     .map_err(|err| err.to_string())?
        //                     .extract()
        //                     .map_err(|err| err.to_string())?;
        //                 Ok(transcription)
        //             })();
        //             // The receiver is maintained by the global state, so if this fails, the app has been closed
        //             let _ = result_tx.send(transcription_res);
        //         }
        //     });
        // }
    });
    // Check if there was a panic (this is long-running, so there has to have been if it's done)
    std::thread::sleep(std::time::Duration::from_secs(3));
    if handle.is_finished() {
        panic!("underlying whisper thread failed to instantiate: static linking has likely failed (please report this as a bug)");
    }

    tauri::Builder::default()
        // We'll always start idle
        .manage(AppState {
            transcription_state: Mutex::new(TranscriptionState::Idle),
            record_file,
            transcribe_tx: Mutex::new(transcription_action_tx),
            transcribe_rx: Arc::new(Mutex::new(transcription_rx)),
        })
        .invoke_handler(tauri::generate_handler![transcribe, record, end_recording])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

/// The current operation in the transcription cycle.
enum TranscriptionState {
    /// We're actively recording new audio.
    Recording(oneshot::Sender<()>),
    /// We're actively transcribing recorded audio.
    Transcribing,
    /// There is audio that has been recorded, but not yet transcribed.
    Recorded,
    /// The app is idle, and waiting for audio input.
    Idle,
}

struct AppState {
    transcription_state: Mutex<TranscriptionState>,
    /// The handle to the file used for recording. When this is dropped (i.e. when the app closes), it
    /// will be automatically dropped by the OS.
    record_file: NamedTempFile,
    /// We need a separate thread for transcription so we can hold the embedded Python
    /// interpreter lock: this sender will indicate when that thread should start transcribing.
    transcribe_tx: Mutex<mpsc::Sender<()>>,
    /// A receiver for the output of the transcription thread. This is a *separate*
    /// tranceiver pair from the
    transcribe_rx: Arc<Mutex<mpsc::Receiver<Result<String, String>>>>,
}

#[tauri::command]
async fn record(state: State<'_, AppState>) -> Result<(), String> {
    let rx = {
        let mut transcription_state = state.transcription_state.lock().unwrap();
        match &*transcription_state {
            TranscriptionState::Transcribing => return Err("currently transcribing".to_string()),
            TranscriptionState::Recording(_) => return Err("already recording".to_string()),
            // Even if we have previous audio that hasn't yet been transcribed, we can override it
            TranscriptionState::Idle | TranscriptionState::Recorded => {
                // We have recorded audio that we can work with
                // Create a new oneshot channel for recording
                let (tx, rx) = oneshot::channel::<()>();
                // The sender will be needed to stop the recording later
                *transcription_state = TranscriptionState::Recording(tx);
                rx
            }
        }
    };

    let path = state.record_file.path().to_string_lossy().to_string();
    tokio::task::spawn_blocking(move || {
        crate::record::start_recording(&path, rx)
    }).await.map_err(|_| String::from("recording thread panicked"))??;

    Ok(())
}

#[tauri::command]
async fn end_recording(state: State<'_, AppState>) -> Result<(), String> {
    let mut transcription_state = state.transcription_state.lock().unwrap();
    let tx = match &*transcription_state {
        TranscriptionState::Recording(_) => {
            // We need the full `tx`, so we'll directly replace the state in here
            let old_state = std::mem::replace(&mut *transcription_state, TranscriptionState::Recorded);
            if let TranscriptionState::Recording(tx) = old_state {
                tx
            }
            else {
                unreachable!()
            }
        },
        _ => return Err("not recording".to_string()),
    };
    // Signal the ongoing recording command to terminate (ignore if the receiver has
    // been dropped, that's probably a messy cleanup)
    let _ = tx.send(());

    Ok(())
}

#[tauri::command]
async fn transcribe(state: State<'_, AppState>) -> Result<String, String> {
    // Assess the transcription lock, dropping it afterward (before an asynchronous wait on the transcription thread)
    let mut transcription_state = state.transcription_state.lock().unwrap();
    match &*transcription_state {
        TranscriptionState::Idle => return Err("no audio recorded yet".to_string()),
        TranscriptionState::Transcribing => return Err("already transcribing".to_string()),
        TranscriptionState::Recording(_) => return Err("currently recording audio, realtime transcription is not yet supported".to_string()),
        TranscriptionState::Recorded => {
            // We have recorded audio that we can work with
            *transcription_state = TranscriptionState::Transcribing;
        }
    };

    // Instruct the transcription thread to transcribe (the thread must be responsive at this stage)
    state.transcribe_tx.lock().unwrap().send(())
        .map_err(|_| "transcription thread unresponsive, please restart the app".to_string())?;

    // And wait for its response
    let res = state.transcribe_rx.lock().unwrap().recv();
    let res = match res {
        Ok(Ok(msg)) => Ok(msg),
        Ok(Err(err)) => Err(err.to_string()),
        // The transcription thread should not close until the whole app does
        Err(_) => Err("transcription thread closed prematurely, please restart the app".to_string()),
    };
    // Release the transcription lock
    *transcription_state = TranscriptionState::Idle;

    res

}
