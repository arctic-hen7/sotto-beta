#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod dictate;
mod errors;
mod record;
mod transcribe;

use crate::dictate::AppState;
use tauri::State;

fn main() {
    tauri::Builder::default()
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![dictate, end_recording])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

// We abstract away the hybrid sync/async dynamics of the underlying dictation mechanism
// (because the frontend caller doesn't care about whether or not the app state has updated,
// and a mutex is locked anyway, so the locking would block until it was ready, so this should
// always work)
#[tauri::command]
async fn dictate(state: State<'_, AppState>) -> Result<String, String> {
    let task_fut = state.dictate().map_err(|e| e.to_string())?;
    task_fut.await.map_err(|e| e.to_string())
}
#[tauri::command]
async fn end_recording(state: State<'_, AppState>) -> Result<(), String> {
    state.end_recording().await.map_err(|e| e.to_string())
}
