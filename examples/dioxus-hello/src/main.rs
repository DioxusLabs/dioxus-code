use dioxus::prelude::*;

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        main {
            style: "padding: 32px; font-family: sans-serif;",
            h1 { "Hello, world!" }
        }
    }
}
