use dioxus::prelude::*;
use dioxus_code::{Code, Theme, code};

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        Code {
            src: code!("/snippets/example.rs"),
            theme: Theme::GITHUB_DARK,
        }
    }
}
