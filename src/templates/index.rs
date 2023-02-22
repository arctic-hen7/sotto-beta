use perseus::prelude::*;
use serde::{Deserialize, Serialize};
use sycamore::prelude::*;

#[auto_scope]
fn index_page<G: Html>(cx: Scope, IndexStateRx { greeting }: &IndexStateRx) -> View<G> {
    let greeting = create_memo(cx, || match &*greeting.get() {
        Ok(greeting) => greeting.to_string(),
        _ => unreachable!(),
    });

    view! { cx,
        // Don't worry, there are much better ways of styling in Perseus!
        div(style = "display: flex; flex-direction: column; justify-content: center; align-items: center; height: 95vh;") {
            h1 { "Welcome to Perseus!" }
            p {
                "This is just an example app. Try changing some code inside "
                code { "src/templates/index.rs" }
                " and you'll be able to see the results here!"
            }
            p { (greeting.get()) }
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
    #[rx(suspense = "get_greeting")]
    greeting: Result<String, SerdeInfallible>,
}

#[browser_only_fn]
async fn get_greeting<'a>(
    _cx: Scope<'a>,
    greeting: &'a RcSignal<Result<String, SerdeInfallible>>,
) -> Result<(), SerdeInfallible> {
    let tauri_greeting = crate::tauri::greet("Sir").await;
    let tauri_greeting = tauri_greeting.as_string().unwrap();
    greeting.set(Ok(tauri_greeting));
    Ok(())
}

#[engine_only_fn]
async fn get_build_state(_: StateGeneratorInfo<()>) -> IndexState {
    IndexState {
        greeting: Ok(String::new()),
    }
}

pub fn get_template<G: Html>() -> Template<G> {
    Template::build("index")
        .view_with_state(index_page)
        .head(head)
        .build_state_fn(get_build_state)
        .build()
}
