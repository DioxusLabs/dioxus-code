use dioxus::prelude::*;
use dioxus_code::{Code, RuntimeCode, Theme};

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
        style { {APP_CSS} }
        main { class: "live-shell",
            section { class: "editor-pane",
                div { class: "pane-head",
                    h1 { "Live Arborium Highlighting" }
                    span { "rust" }
                }
                textarea {
                    class: "code-input",
                    spellcheck: false,
                    value: "{source}",
                    oninput: move |event| source.set(event.value()),
                }
            }
            section { class: "preview-pane",
                div { class: "pane-head",
                    h2 { "Preview" }
                    span { "runtime" }
                }
                Code {
                    src: RuntimeCode::new(source()).with_language("rust"),
                    theme: Theme::TOKYO_NIGHT,
                }
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
  background:
    linear-gradient(135deg, rgba(14, 27, 43, 0.92), rgba(18, 18, 27, 0.96)),
    repeating-linear-gradient(90deg, rgba(255,255,255,0.025) 0 1px, transparent 1px 72px);
  color: #e8eef7;
  font-family: Avenir Next, Gill Sans, Trebuchet MS, sans-serif;
}

.live-shell {
  display: grid;
  grid-template-columns: minmax(320px, 0.95fr) minmax(320px, 1.05fr);
  gap: 22px;
  min-height: 100vh;
  padding: 28px;
  box-sizing: border-box;
}

.editor-pane,
.preview-pane {
  display: grid;
  grid-template-rows: auto minmax(0, 1fr);
  min-width: 0;
}

.pane-head {
  display: flex;
  align-items: end;
  justify-content: space-between;
  gap: 16px;
  padding: 0 2px 12px;
}

h1,
h2 {
  margin: 0;
  font-size: clamp(20px, 3vw, 34px);
  font-weight: 700;
  letter-spacing: 0;
}

h2 {
  font-size: clamp(18px, 2vw, 24px);
}

.pane-head span {
  border: 1px solid rgba(232, 238, 247, 0.28);
  color: #9fb4ce;
  font: 700 12px ui-monospace, SFMono-Regular, Menlo, monospace;
  letter-spacing: 0;
  padding: 5px 8px;
  text-transform: uppercase;
}

.code-input {
  resize: none;
  min-height: 0;
  border: 1px solid rgba(159, 180, 206, 0.32);
  border-radius: 8px;
  background: #f6f2ee;
  color: #1d2633;
  box-shadow: 0 18px 70px rgba(0, 0, 0, 0.32);
  box-sizing: border-box;
  font: 15px/1.55 ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
  outline: none;
  padding: 18px;
  tab-size: 4;
}

.code-input:focus {
  border-color: #7aa2f7;
  box-shadow: 0 18px 70px rgba(0, 0, 0, 0.32), 0 0 0 3px rgba(122, 162, 247, 0.22);
}

.preview-pane .dxc {
  min-height: 0;
  box-shadow: 0 18px 70px rgba(0, 0, 0, 0.38);
}

@media (max-width: 820px) {
  .live-shell {
    grid-template-columns: 1fr;
    grid-template-rows: minmax(320px, 48vh) minmax(260px, auto);
    padding: 18px;
  }
}
"#;
