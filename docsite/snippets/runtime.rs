use dioxus::prelude::*;
use dioxus_code::{Code, CodeTheme, Language, SourceCode, Theme};

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    let source = use_signal(|| "fn main() {}".to_string());

    rsx! {
        Code {
            src: SourceCode::new(Language::Rust, source()),
            theme: CodeTheme::system(Theme::GITHUB_LIGHT, Theme::GITHUB_DARK),
        }
    }
}
