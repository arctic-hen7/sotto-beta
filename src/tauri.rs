//! This module is responsible for providing the FFI interface to our Tauri commands (which are written in Rust, exposed through a JS
//! FFI interface, and then re-accessed through Rust --- efficiency!).

use wasm_bindgen::prelude::*;

#[cfg(client)]
#[wasm_bindgen(module = "/src/tauri_glue.js")]
extern "C" {
    pub async fn greet(name: &str) -> JsValue;
}
