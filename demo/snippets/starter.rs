use dioxus::prelude::*;
use dioxus_code::{Code, Theme, code};

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        div { class: "app",
            Code {
                src: code!("/snippets/demo.rs"),
                theme: Theme::GITHUB_DARK,
            }
        }
    }
}
