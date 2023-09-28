use perseus::prelude::*;
use serde::{Deserialize, Serialize};
use sycamore::prelude::*;

#[auto_scope]
fn index_page<G: Html>(cx: Scope, state: &IndexStateRx) -> View<G> {
    let help_shown = create_signal(cx, false);

    // Displays the appropriate 'big button' according to the current app state
    let button_view = create_memo(cx, move || {
        let sotto_state = state.state.get();
        match &*sotto_state {
            // Record button
            SottoState::Ready => view! {
                cx,
                button(
                    on:click = move |_| {
                        #[cfg(client)]
                        state.dictate(cx);
                    },
                    class = "flex justify-center items-center rounded-full h-96 w-96 border border-neutral-400 hover:border-opacity-0 transition-all duration-250"
                ) {
                    span(class = "bg-red-400 h-[93%] w-[93%] rounded-full hover:h-full hover:w-full transition-all") {}
                }
            },
            // Recording indicator that allows ending the recording
            SottoState::Recording => view! {
                cx,
                button(
                    on:click = move |_| {
                        #[cfg(client)]
                        state.end_recording(cx);
                    },
                    class = "relative flex justify-center items-center rounded-full h-96 w-96 text-red-400"
                ) {
                    svg(class = "animate-spin z-50", xmlns = "http://www.w3.org/2000/svg", viewBox = "0 0 100 100", width = "100%", height = "100%") {
                        circle(cx = "50", cy = "50", r = "45", stroke = "currentColor", stroke-width = "3", fill = "transparent", stroke-dasharray = "90 1000", stroke-linecap = "round") {}
                    }
                    span(class = "absolute bg-red-400 h-[83%] w-[83%] rounded-full transition-all") {}
                    svg(class = "absolute fill-white", xmlns = "http://www.w3.org/2000/svg", viewBox = "0 0 100 100", width = "100%", height = "100%") {
                        rect(x = "30", y = "30", width = "40", height = "40") {}
                    }
                }
            },
            // Transcription indicator
            SottoState::Transcribing => view! {
                cx,
                // *Not* a button, this one is just an indicator!
                div(
                    class = "relative flex justify-center items-center rounded-full h-96 w-96 text-emerald-400"
                ) {
                    svg(class = "z-50 animate-spin", xmlns = "http://www.w3.org/2000/svg", viewBox = "0 0 100 100", width = "100%", height = "100%") {
                        circle(cx = "50", cy = "50", r = "45", stroke = "currentColor", stroke-width = "3", fill = "transparent", stroke-dasharray = "90 1000", stroke-linecap = "round") {}
                        // Alignments eyeballed
                        circle(cx = "50", cy = "50", r = "20", stroke = "white", stroke-width = "3", fill = "transparent", stroke-dasharray = "20 1000", stroke-dashoffset = "-11", stroke-linecap = "round") {}
                    }
                    span(class = "absolute bg-emerald-400 h-[83%] w-[83%] rounded-full transition-all") {}
                }
            },
            SottoState::Err(err) => {
                let err = err.to_string();
                view! {
                    cx,
                    div(class = "flex flex-col justify-center align-items") {
                        // Not a button, the user should manually restart the app
                        div(
                            class = "relative flex justify-center items-center rounded-full h-96 w-96 text-red-700"
                        ) {
                            span(class = "absolute bg-red-700 h-full w-full rounded-full") {}
                            svg(class = "absolute fill-white", xmlns = "http://www.w3.org/2000/svg", viewBox = "0 0 100 100", width = "70%", height = "70%") {
                                rect(x = "42.5", y = "15", width = "15", height = "45", fill = "white") {}
                                circle(cx = "50", cy = "80", r = "10", fill = "white") {}
                            }
                        }
                        p(class = "text-xl max-w-sm text-center text-red-800 mt-4") { "An error has occurred. Please take a photo of this error so we can understand it better, and then restart the app. Make sure to copy any transcribed text first!" }
                        div(class = "bg-red-400 rounded-lg max-w-md text-white p-6 text-lg mt-4") {
                            p(class = "break-words") { (format!("Error: '{}'", err)) }
                        }
                    }

                }
            }
        }
    });

    view! { cx,
        div(class = "w-full h-screen flex flex-col justify-center items-center") {
            div(class = "flex flex-row justify-center items-center w-full") {
                (*button_view.get())
                textarea(
                    class = "p-4 mx-4 border border-black text-4xl h-96 w-1/2 resize",
                    bind:value = state.text,
                    placeholder = "Try recording some audio, and, when it's been transcribed, the text will appear here!"
                ) {}
            }
            button(
                on:click = |_| {
                    help_shown.set(!*help_shown.get_untracked());
                },
                class = "mt-8 text-xl p-2 bg-red-400 text-white rounded-md hover:bg-red-700 transition-colors"
            ) { "Help!" }
            div(class = format!(
                "text-2xl mx-4 {}",
                if *help_shown.get() {
                    "block"
                } else {
                    "hidden"
                }
            )) {
                ol(class = "ml-12 list-decimal") {
                    li { "Press the big red button." }
                    li { "Dictate what you want." }
                    li { "Press the big red button again to stop recording." }
                    li { "Press the big green button to transcribe, and wait until the red button appears again." }
                    li { "Edit your text manually in the text-area to the right." }
                }
                p { "If you encounter a dark red circle with an excalamation mark, it means there's been an error. That page will tell you what to do." }
            }
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
    /// The text that has been transcribed. This will be accumulated as the user transcribes
    /// more and more speech.
    text: String,
    /// The system state.
    state: SottoState,
}

/// The state of the transcription and recording system.
///
/// This is analogous in some ways to the similar state `enum` within the
/// backend of Sotto.
#[derive(Serialize, Deserialize, Clone)]
enum SottoState {
    /// We're actively recording, and are ready to stop recording at any moment.
    Recording,
    /// We're transcribing some text, and waiting for Whisper to finish.
    Transcribing,
    /// An error occurred somewhere in our interactions with Tauri, which should be displayed
    /// to the user.
    Err(String),
    /// We're ready to record some new text.
    ///
    /// This is both the initial state and the state after transcription has been completed.
    Ready,
}
#[cfg(client)]
impl IndexStateRx {
    /// Extends the currently known transcription with further text.
    ///
    /// This will extend the existing text with a new paragraph, to make delimiting clearer. It's also
    /// much easier for a human to remove paragraphing than to add it!
    fn extend_transcription<'a>(&'a self, cx: Scope<'a>, new_text: String) {
        // Whisper sometimes adds some padding
        let new_text = new_text.trim();
        let old_text = self.text.get();
        let updated_text = if old_text.is_empty() {
            // No new paragraph if this is the first thing
            new_text.to_string()
        } else {
            format!("{}\n\n{}", old_text, new_text)
        };
        self.text.set(updated_text);
    }
    /// Instructs Tauri to begin the recording process.
    fn dictate<'a>(&'a self, cx: Scope<'a>) {
        self.state.set(SottoState::Recording);

        // IMPORTANT: The Tauri function that begins the recording spawns a blocking thread that waits for
        // the termination signal, meaning the start operation continues until we stop it. That means this
        // future will continue running until we terminate the recording.
        //
        // TODO Timer to make sure we don't end up recording hours on end of silence?
        spawn_local_scoped(cx, async move {
            // This is a future which will return the transcribed text when it's done
            let res = crate::tauri::dictate().await;
            match res {
                Ok(new_text) => {
                    let new_text = new_text.as_string().unwrap();
                    perseus::web_log!("{}", &new_text);
                    self.extend_transcription(cx, new_text);
                    self.state.set(SottoState::Ready);
                }
                Err(err) => self.state.set(SottoState::Err(err.as_string().unwrap())),
            };
        });
    }
    /// Instructs Tauri to stop recording audio.
    fn end_recording<'a>(&'a self, cx: Scope<'a>) {
        // This will resolve instantly, but Tauri does everything asynchronously, so we still need it
        spawn_local_scoped(cx, async move {
            let res = crate::tauri::end_recording().await;
            match res {
                Ok(_) => self.state.set(SottoState::Transcribing),
                Err(err) => self.state.set(SottoState::Err(err.as_string().unwrap())),
            };
        });
    }
}

#[engine_only_fn]
async fn get_build_state(_: StateGeneratorInfo<()>) -> IndexState {
    IndexState {
        text: String::new(),
        state: SottoState::Ready,
    }
}

pub fn get_template<G: Html>() -> Template<G> {
    Template::build("index")
        .view_with_state(index_page)
        .head(head)
        .build_state_fn(get_build_state)
        .build()
}
