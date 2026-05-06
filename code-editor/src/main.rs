use dioxus::prelude::*;
use dioxus_code::Theme;
use dioxus_code_editor::{CodeEditor, Language};

const DEMO_CSS: Asset = asset!("/assets/demo.css");

const STARTER: &str = r#"pub fn luminance(rgb: (u8, u8, u8)) -> f32 {
    let (r, g, b) = rgb;
    0.2126 * r as f32 + 0.7152 * g as f32 + 0.0722 * b as f32
}
"#;

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    let mut source = use_signal(|| STARTER.to_string());

    rsx! {
        document::Stylesheet { href: DEMO_CSS }
        main { class: "shell",
            section { class: "toolbar",
                h1 { "Code editor component" }
                span { "rust" }
            }
            CodeEditor {
                value: source(),
                language: Language::Rust,
                theme: Theme::TOKYO_NIGHT,
                oninput: move |value| source.set(value),
            }
        }
    }
}
