use dioxus::prelude::*;
use dioxus_code::{CodeSpan, HighlightSpan, IntoTree, RuntimeCode, Theme};

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
    let theme = Theme::TOKYO_NIGHT;
    let tree = RuntimeCode::new(source()).with_language("rust").into_tree();
    let lines = editor_lines(tree.source(), tree.spans());
    let line_count = lines.len();

    rsx! {
        document::Stylesheet { href: theme.asset() }
        style { {APP_CSS} }
        main { class: "shell",
            section { class: "toolbar",
                h1 { "Contenteditable highlighting" }
                span { "rust" }
            }
            div { class: "editor-shell {theme.class()}",
                div { class: "gutter-layer", aria_hidden: "true",
                    for index in 0..line_count {
                        div { class: "gutter-line", "{index + 1}" }
                    }
                }
                div { class: "code-viewport",
                    div { class: "highlight-layer", aria_hidden: "true",
                        for line in lines.into_iter() {
                            div { class: "highlight-line",
                                for segment in line {
                                    if let Some(tag) = segment.tag {
                                        CodeSpan {
                                            text: segment.text.to_string(),
                                            tag,
                                        }
                                    } else {
                                        span { "{segment.text}" }
                                    }
                                }
                            }
                        }
                    }
                    div {
                        class: "input-layer",
                        contenteditable: "plaintext-only",
                        spellcheck: false,
                        role: "textbox",
                        "aria-multiline": "true",
                        oninput: move |event| source.set(event.value()),
                        {STARTER}
                    }
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Segment<'a> {
    text: &'a str,
    tag: Option<&'static str>,
}

fn editor_lines<'a>(source: &'a str, spans: &[HighlightSpan]) -> Vec<Vec<Segment<'a>>> {
    let mut lines = vec![Vec::new()];

    for segment in segments(source, spans) {
        push_line_segments(&mut lines, segment);
    }

    lines
}

fn segments<'a>(source: &'a str, spans: &[HighlightSpan]) -> Vec<Segment<'a>> {
    if spans.is_empty() {
        return vec![Segment {
            text: source,
            tag: None,
        }];
    }

    let mut spans = spans.to_vec();
    spans.sort_by(|a, b| a.start.cmp(&b.start).then_with(|| b.end.cmp(&a.end)));

    let mut events = Vec::with_capacity(spans.len() * 2);
    for (index, span) in spans.iter().enumerate() {
        events.push((span.start, true, index));
        events.push((span.end, false, index));
    }
    events.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.cmp(&b.1)));

    let mut segments = Vec::new();
    let mut last_pos = 0;
    let mut stack: Vec<usize> = Vec::new();

    for (pos, is_start, span_index) in events {
        let pos = pos as usize;
        if pos > last_pos && pos <= source.len() {
            segments.push(Segment {
                text: &source[last_pos..pos],
                tag: stack.last().map(|&i| spans[i].tag),
            });
            last_pos = pos;
        }

        if is_start {
            stack.push(span_index);
        } else if let Some(index) = stack.iter().rposition(|&i| i == span_index) {
            stack.remove(index);
        }
    }

    if last_pos < source.len() {
        segments.push(Segment {
            text: &source[last_pos..],
            tag: stack.last().map(|&i| spans[i].tag),
        });
    }

    segments
}

fn push_line_segments<'a>(lines: &mut Vec<Vec<Segment<'a>>>, segment: Segment<'a>) {
    let mut text = segment.text;

    loop {
        if let Some(newline) = text.find('\n') {
            let before_newline = &text[..newline];
            if !before_newline.is_empty() {
                lines.last_mut().unwrap().push(Segment {
                    text: before_newline,
                    tag: segment.tag,
                });
            }
            lines.push(Vec::new());
            text = &text[newline + 1..];
        } else {
            if !text.is_empty() {
                lines.last_mut().unwrap().push(Segment {
                    text,
                    tag: segment.tag,
                });
            }
            break;
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
  grid-template-rows: auto minmax(0, 1fr);
  gap: 16px;
  min-height: 100vh;
  padding: 24px;
}

.toolbar {
  align-items: end;
  display: flex;
  justify-content: space-between;
}

h1 {
  font-size: 22px;
  letter-spacing: 0;
  margin: 0;
}

.toolbar span {
  border: 1px solid rgba(217, 225, 238, 0.28);
  color: #94a3b8;
  font: 700 12px ui-monospace, SFMono-Regular, Menlo, monospace;
  padding: 5px 8px;
  text-transform: uppercase;
}

.editor-shell {
  border-radius: 8px;
  box-sizing: border-box;
  display: grid;
  font: 15px/1.55 ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
  grid-template-columns: max-content minmax(0, 1fr);
  min-height: 0;
  overflow: auto;
  tab-size: 4;
}

.code-viewport {
  box-sizing: border-box;
  min-height: 100%;
  min-width: 0;
  position: relative;
}

.gutter-layer,
.highlight-layer,
.input-layer {
  box-sizing: border-box;
  min-height: 100%;
}

.highlight-layer,
.input-layer {
  padding: 14px 18px 14px 0;
  white-space: pre;
}

.gutter-layer {
  color: var(--muted, currentColor);
  padding: 14px 0;
  pointer-events: none;
  user-select: none;
}

.highlight-layer {
  pointer-events: none;
}

.highlight-layer span {
  font-style: normal !important;
  font-weight: inherit !important;
}

.input-layer {
  caret-color: #e8eef7;
  color: transparent;
  inset: 0;
  outline: none;
  position: absolute;
  z-index: 1;
}

.input-layer::selection {
  background: rgba(122, 162, 247, 0.34);
  color: transparent;
}

.editor-shell:focus-within {
  box-shadow: 0 0 0 3px rgba(122, 162, 247, 0.22);
}

.gutter-line,
.highlight-line {
  box-sizing: border-box;
  min-height: 1.55em;
}

.gutter-line {
  min-width: 4ch;
  padding: 0 14px;
  text-align: right;
}
"#;
