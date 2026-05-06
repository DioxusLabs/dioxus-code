use dioxus::prelude::*;
use dioxus_code::{Code, CodeTheme, SourceCode, Theme};

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    let source = use_signal(|| "fn main() {}".to_string());

    rsx! {
        Code {
            src: SourceCode::new(source()).with_language("rust"),
            theme: CodeTheme::system(Theme::GITHUB_LIGHT, Theme::GITHUB_DARK),
        }
    }
}
