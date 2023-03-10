#[cfg(client)]
mod tauri;
mod templates;

use perseus::prelude::*;
use sycamore::prelude::*;

#[perseus::main_export]
pub fn main<G: Html>() -> PerseusApp<G> {
    PerseusApp::new()
        .template(crate::templates::index::get_template())
        .index_view(|cx| {
            view! {
                cx,
                html {
                    head {
                        link(rel = "stylesheet", href = ".perseus/static/tailwind.css")
                    }
                    body {
                        PerseusRoot {}
                    }
                }
            }
        })
}
