#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod record;

use std::sync::Mutex;
use tauri::State;
use tokio::sync::oneshot::{channel, Sender};

fn main() {
    tauri::Builder::default()
        // We'll always start idle
        .manage(AppState { transcription_state: Mutex::new(TranscriptionState::Idle) })
        .invoke_handler(tauri::generate_handler![transcribe, record, end_recording])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

/// The current operation in the transcription cycle.
enum TranscriptionState {
    /// We're actively recording new audio.
    Recording(Sender<()>),
    /// We're actively transcribing recorded audio.
    Transcribing,
    /// There is audio that has been recorded, but not yet transcribed.
    Recorded,
    /// The app is idle, and waiting for audio input.
    Idle,
}

struct AppState {
    transcription_state: Mutex<TranscriptionState>,
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
                let (tx, rx) = channel::<()>();
                // The sender will be needed to stop the recording later
                *transcription_state = TranscriptionState::Recording(tx);
                rx
            }
        }
    };

    tokio::task::spawn_blocking(move || {
        crate::record::start_recording("../record.wav", rx)
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
    use pyo3::{Python, PyErr, types::PyModule};

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

    let res: Result<String, PyErr> = Python::with_gil(|py| {
        let py_code = include_str!("transcribe.py");
        // Hilariously, Python won't let us call this `whisper`
        let whisper = PyModule::from_code(py, py_code, "transcribe.py", "transcribe")?;
        let transcription: String = whisper.getattr("transcribe")?.call1(("../record.wav",))?.extract()?;

        Ok(transcription)
    });
    let res = match res {
        Ok(msg) => Ok(msg),
        Err(err) => Err(err.to_string()),
    };
    // Release the transcription lock
    *transcription_state = TranscriptionState::Idle;

    res
}
