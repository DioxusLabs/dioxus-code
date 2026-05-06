use dioxus::prelude::*;
use dioxus_code::{CodeTheme, Theme};
use dioxus_code_editor::{CodeEditor, Language};

const STARTER: &str = r#"fn fizzbuzz(n: u32) -> String {
    match (n % 3, n % 5) {
        (0, 0) => "FizzBuzz".to_string(),
        (0, _) => "Fizz".to_string(),
        (_, 0) => "Buzz".to_string(),
        _ => n.to_string(),
    }
}
"#;

const THEMES: &[(&str, Theme)] = &[
    ("Tokyo Night", Theme::TOKYO_NIGHT),
    ("GitHub Light", Theme::GITHUB_LIGHT),
    ("GitHub Dark", Theme::GITHUB_DARK),
    ("Catppuccin Mocha", Theme::CATPPUCCIN_MOCHA),
    ("Dracula", Theme::DRACULA),
    ("Solarized Light", Theme::SOLARIZED_LIGHT),
];

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    let mut source = use_signal(|| STARTER.to_string());
    let mut theme_index = use_signal(|| 0usize);
    let mut line_numbers = use_signal(|| true);
    let mut read_only = use_signal(|| false);

    let (theme_label, theme) = THEMES[theme_index()];

    rsx! {
        style { {APP_CSS} }
        main { class: "shell",
            header { class: "topbar",
                div {
                    h1 { "dioxus-code-editor" }
                    p { "Type below to see live syntax highlighting." }
                }
                div { class: "controls",
                    label {
                        span { "Theme" }
                        select {
                            value: "{theme_index()}",
                            onchange: move |event| {
                                if let Ok(index) = event.value().parse::<usize>() {
                                    theme_index.set(index);
                                }
                            },
                            for (i, (name, _)) in THEMES.iter().enumerate() {
                                option { value: "{i}", selected: i == theme_index(), "{name}" }
                            }
                        }
                    }
                    label { class: "checkbox",
                        input {
                            r#type: "checkbox",
                            checked: line_numbers(),
                            onchange: move |event| line_numbers.set(event.checked()),
                        }
                        span { "Line numbers" }
                    }
                    label { class: "checkbox",
                        input {
                            r#type: "checkbox",
                            checked: read_only(),
                            onchange: move |event| read_only.set(event.checked()),
                        }
                        span { "Read only" }
                    }
                    button {
                        r#type: "button",
                        onclick: move |_| source.set(STARTER.to_string()),
                        "Reset"
                    }
                }
            }
            section { class: "editor-frame",
                div { class: "frame-bar",
                    span { "fizzbuzz.rs" }
                    span { "{theme_label}" }
                }
                CodeEditor {
                    value: source(),
                    language: Language::Rust,
                    theme: CodeTheme::fixed(theme),
                    line_numbers: line_numbers(),
                    read_only: read_only(),
                    aria_label: "Rust source editor",
                    placeholder: "Type Rust code...",
                    class: "example-editor",
                    oninput: move |value| source.set(value),
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
  background: #10131a;
  color: #d9e1ee;
  font-family: Avenir Next, Gill Sans, Trebuchet MS, sans-serif;
}

.shell {
  box-sizing: border-box;
  display: grid;
  gap: 20px;
  grid-template-rows: auto minmax(0, 1fr);
  min-height: 100vh;
  padding: 28px;
}

.topbar {
  align-items: end;
  display: flex;
  flex-wrap: wrap;
  gap: 24px;
  justify-content: space-between;
}

.topbar h1 {
  font-size: 24px;
  letter-spacing: 0;
  margin: 0 0 4px;
}

.topbar p {
  color: #94a3b8;
  margin: 0;
  font-size: 14px;
}

.controls {
  align-items: center;
  display: flex;
  flex-wrap: wrap;
  gap: 16px;
}

.controls label {
  align-items: center;
  display: inline-flex;
  font-size: 13px;
  gap: 8px;
}

.controls label > span {
  color: #94a3b8;
  font: 700 11px ui-monospace, SFMono-Regular, Menlo, monospace;
  letter-spacing: 0.04em;
  text-transform: uppercase;
}

.controls select,
.controls button {
  background: #1b2030;
  border: 1px solid rgba(217, 225, 238, 0.18);
  border-radius: 6px;
  color: #e8eef7;
  font: 13px ui-monospace, SFMono-Regular, Menlo, monospace;
  padding: 6px 10px;
}

.controls button {
  cursor: pointer;
}

.controls button:hover {
  border-color: #7aa2f7;
}

.controls .checkbox span {
  color: #d9e1ee;
  font-weight: 500;
  letter-spacing: 0;
  text-transform: none;
}

.editor-frame {
  border: 1px solid rgba(217, 225, 238, 0.18);
  border-radius: 10px;
  display: grid;
  grid-template-rows: auto minmax(0, 1fr);
  min-height: 0;
  overflow: hidden;
}

.frame-bar {
  align-items: center;
  background: #181c27;
  border-bottom: 1px solid rgba(217, 225, 238, 0.1);
  display: flex;
  font: 700 12px ui-monospace, SFMono-Regular, Menlo, monospace;
  justify-content: space-between;
  padding: 10px 14px;
}

.frame-bar span:first-child {
  color: #d9e1ee;
}

.frame-bar span:last-child {
  color: #94a3b8;
  text-transform: uppercase;
}

.example-editor {
  --dxc-editor-caret: #e8eef7;
  --dxc-editor-focus-ring: 0 0 0 3px rgba(122, 162, 247, 0.22);
  min-height: 0;
}
"#;
