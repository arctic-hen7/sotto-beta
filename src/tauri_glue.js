// This file contains glue code needed to get the Tauri commands to work with Perseus.

const invoke = window.__TAURI__.tauri.invoke;

export async function transcribe() {
  return await invoke("transcribe");
}
export async function record() {
  return await invoke("record");
}
export async function end_recording() {
  return await invoke("end_recording");
}
