// This file contains glue code needed to get the Tauri commands to work with Perseus.

const invoke = window.__TAURI__.tauri.invoke;

export async function greet(name) {
  return await invoke("greet", {name: name});
}
