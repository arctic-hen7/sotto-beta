#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod dictate;
mod errors;
mod model;
mod record;
mod transcribe;

use crate::dictate::AppState;
use tauri::State;

fn main() {
    tauri::Builder::default()
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![dictate, end_recording])
        .run(tauri::generate_context!())
        // Critical error, we definitionally can't proceed
        .expect("failed to start tauri application");
}

// We abstract away the hybrid sync/async dynamics of the underlying dictation mechanism
// (because the frontend caller doesn't care about whether or not the app state has updated,
// and a mutex is locked anyway, so the locking would block until it was ready, so this should
// always work)
#[tauri::command]
async fn dictate(state: State<'_, AppState>) -> Result<String, String> {
    let task_fut = state.dictate().map_err(|e| format!("{e:?}"))?;
    task_fut.await.map_err(|e| format!("{e:?}"))
}
#[tauri::command]
async fn end_recording(state: State<'_, AppState>) -> Result<(), String> {
    state.end_recording().await.map_err(|e| format!("{e:?}"))
}
