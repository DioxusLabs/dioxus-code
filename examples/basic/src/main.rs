use dioxus::prelude::*;
use dioxus_code::{Code, Language, SourceCode, Theme, code};

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        main {
            style: "display:grid;gap:20px;padding:32px;max-width:900px;margin:0 auto;",
            Code {
                src: code!("/snippets/demo.rs"),
                theme: Theme::RUSTDOC_AYU,
            }
            Code {
                src: SourceCode::new("fn main() {\n    println!(\"runtime\");\n}")
                    .with_language(Language::Rust),
                theme: Theme::GITHUB_LIGHT,
            }
        }
    }
}
