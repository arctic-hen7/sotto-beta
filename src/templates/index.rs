use perseus::prelude::*;
use serde::{Deserialize, Serialize};
use sycamore::prelude::*;

#[auto_scope]
fn index_page<G: Html>(cx: Scope, IndexStateRx { text }: &IndexStateRx) -> View<G> {
    let transcription_state = create_signal(cx, TranscriptionState::None);
    let transcription_state_view = create_memo(cx, move || {
        let state = transcription_state.get();
        match &*state {
            TranscriptionState::Ok(new_text) => {
                // Whisper sometimes adds some padding
                let new_text = new_text.trim();
                let old_text = text.get();
                let updated_text = format!("{} {}", old_text, new_text);
                text.set(updated_text);
                View::empty()
            },
            TranscriptionState::None => View::empty(),
            TranscriptionState::Loading => view! {
                cx,
                p { "Loading..." }
            },
            TranscriptionState::Err(err) => {
                let err = err.to_string();
                view! {
                    cx,
                    p(style = "color: red;") { (format!("An error occurred: '{}'.", err)) }
                }
            }
        }
    });

    view! { cx,
        div(class = "w-full h-screen flex flex-col justify-center items-center") {
            button(
                class = "",
                on:click = move |_| {
                    #[cfg(client)]
                    transcribe(cx, &transcription_state);
                }
            ) { "Transcribe" }
            textarea(class = "border", bind:value = text) {}
            (*transcription_state_view.get())
        }
    }
}

#[engine_only_fn]
fn head(cx: Scope) -> View<SsrNode> {
    view! { cx,
        title { "Welcome to Perseus!" }
    }
}

#[derive(Serialize, Deserialize, Clone, ReactiveState)]
#[rx(alias = "IndexStateRx")]
struct IndexState {
    /// The text that has been transcribed.
    text: String,
}

enum TranscriptionState {
    Ok(String),
    Loading,
    Err(String),
    None,
}

/// Instructs Tauri to begin the transcription process with whatever audio has been recorded.
/// This manages state and asynchronicity on its own.
#[browser_only_fn]
fn transcribe<'a>(
    cx: Scope<'a>,
    state: &'a Signal<TranscriptionState>,
) {
    state.set(TranscriptionState::Loading);

    spawn_local_scoped(cx, async move {
        let res = crate::tauri::transcribe().await;
        match res {
            Ok(transcription) => state.set(TranscriptionState::Ok(
                transcription
                    .as_string()
                    .unwrap()
            )),
            Err(err) => state.set(TranscriptionState::Err(
                err
                    .as_string()
                    .unwrap()
            )),
        };
    });
}

#[engine_only_fn]
async fn get_build_state(_: StateGeneratorInfo<()>) -> IndexState {
    IndexState {
        text: String::new(),
    }
}

pub fn get_template<G: Html>() -> Template<G> {
    Template::build("index")
        .view_with_state(index_page)
        .head(head)
        .build_state_fn(get_build_state)
        .build()
}
