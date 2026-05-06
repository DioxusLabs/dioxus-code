use dioxus::prelude::*;
use dioxus_code::advanced::HighlightedSource;
use dioxus_code::{Code, CodeOptions, Theme, code};

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    const CODE: HighlightedSource = code!(
        "/snippets/demo.rs",
        CodeOptions::builder().with_language("rust")
    );
    rsx! {
        style { {APP_CSS} }
        main { class: "macro-only-shell",
            header {
                h1 { "Macro-only highlighting" }
                p { "This example embeds parsed Arborium spans at compile time through code!()." }
            }
            Code {
                src: CODE,
                theme: Theme::RUSTDOC_AYU,
            }
        }
    }
}

const APP_CSS: &str = r#"
html, body, #main {
  margin: 0;
  min-height: 100%;
}

body {
  background: #16181d;
  color: #e9edf4;
  font-family: Avenir Next, Gill Sans, Trebuchet MS, sans-serif;
}

.macro-only-shell {
  display: grid;
  gap: 18px;
  max-width: 880px;
  margin: 0 auto;
  padding: 42px 22px;
}

h1 {
  margin: 0;
  font-size: 34px;
  letter-spacing: 0;
}

p {
  margin: 8px 0 0;
  color: #9aa7b8;
}
"#;
