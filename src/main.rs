#[cfg(client)]
mod tauri;
mod templates;

use perseus::prelude::*;

#[perseus::main_export]
pub fn main<G: Html>() -> PerseusApp<G> {
    PerseusApp::new().template(crate::templates::index::get_template())
}
