#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use std::sync::Mutex;
use tauri::State;

fn main() {
    tauri::Builder::default()
        // We'll always start idle
        .manage(AppState { transcription_state: Mutex::new(TranscriptionState::Recorded) })
        .invoke_handler(tauri::generate_handler![transcribe])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

/// The current operation in the transcription cycle.
enum TranscriptionState {
    /// We're actively recording new audio.
    Recording,
    /// We're actively transcribing recorded audio.
    Transcribing,
    /// There is audio that has been recorded, but not yet transcribed.
    Recorded,
    /// The app is idle, and waiting for audio input.
    Idle,
}

struct AppState {
    transcription_state: Mutex<TranscriptionState>
}

#[tauri::command]
async fn transcribe(state: State<'_, AppState>) -> Result<String, String> {
    use pyo3::{Python, PyErr, types::{PyModule, IntoPyDict}};

    let mut transcription_state = state.transcription_state.lock().unwrap();
    match &*transcription_state {
        TranscriptionState::Idle => return Err("no audio recorded yet".to_string()),
        TranscriptionState::Transcribing => return Err("already transcribing".to_string()),
        TranscriptionState::Recording => return Err("currently recording audio, realtime transcription is not yet supported".to_string()),
        TranscriptionState::Recorded => {
            // We have recorded audio that we can work with
            *transcription_state = TranscriptionState::Transcribing;
        }
    };

    let res: Result<String, PyErr> = Python::with_gil(|py| {
        let py_code = include_str!("transcribe.py");
        // Hilariously, Python won't let us call this `whisper`
        let whisper = PyModule::from_code(py, py_code, "transcribe.py", "transcribe")?;
        let transcription: String = whisper.getattr("transcribe")?.call1(("../test2.wav",))?.extract()?;

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
