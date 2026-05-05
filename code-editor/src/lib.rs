use dioxus::prelude::*;
use dioxus_code::{CodeSpan, HighlightSpan, IntoTree, SourceCode, Theme};
use std::{cell::RefCell, rc::Rc};

/// Base stylesheet injected by [`CodeEditor`].
pub const CODE_EDITOR_CSS: Asset = asset!("/assets/dioxus-code-editor.css");

#[derive(Props, Clone, PartialEq)]
pub struct CodeEditorProps {
    /// The current editor contents.
    #[props(into)]
    pub value: String,
    /// Optional Arborium language hint, for example `"rust"`.
    #[props(into, default)]
    pub language: String,
    /// Optional file name used for language detection when `language` is empty.
    #[props(into, default)]
    pub name: String,
    /// Syntax theme shared with `dioxus-code`.
    #[props(default)]
    pub theme: Theme,
    /// Show a gutter with one-based line numbers.
    #[props(default = true)]
    pub line_numbers: bool,
    /// Disable editing while preserving syntax highlighting and text selection.
    #[props(default = false)]
    pub read_only: bool,
    /// Forward spellcheck to the contenteditable input layer.
    #[props(default = false)]
    pub spellcheck: bool,
    /// Accessible label for the editor textbox.
    #[props(into, default = "Code editor")]
    pub aria_label: String,
    /// Placeholder shown only while `value` is empty.
    #[props(into, default)]
    pub placeholder: String,
    /// Extra class names appended to the editor root.
    #[props(into, default)]
    pub class: String,
    /// Called with the full editor text after each input event.
    #[props(default = EventHandler::new(|_: String| {}))]
    pub oninput: EventHandler<String>,
}

/// Editable syntax-highlighted code surface.
///
/// The component is controlled by `value`; update that value from `oninput` to
/// keep the highlight layer and editable layer in sync.
#[component]
pub fn CodeEditor(props: CodeEditorProps) -> Element {
    let input_sync = use_hook(|| {
        Rc::new(RefCell::new(InputSyncState {
            rendered_value: props.value.clone(),
            last_local_value: props.value.clone(),
            version: 0,
        }))
    });

    let mut source_code = SourceCode::new(props.value.clone());
    if !props.language.is_empty() {
        source_code = source_code.with_language(props.language.clone());
    }
    if !props.name.is_empty() {
        source_code = source_code.with_name(props.name.clone());
    }

    let tree = source_code.into_tree();
    let lines = editor_lines(tree.source(), tree.spans());
    let line_count = lines.len();
    let theme_asset = props.theme.asset();
    let theme_key = props.theme.name();
    let class = editor_class(props.theme, props.line_numbers, &props.class);
    let (input_value, input_version) = synced_input_value(&input_sync, &props.value);
    let contenteditable = if props.read_only {
        "false"
    } else {
        "plaintext-only"
    };
    let readonly = props.read_only.then_some("true");

    rsx! {
        {rsx!{document::Stylesheet { key: "{theme_key}", href: theme_asset }}}
        document::Stylesheet { href: CODE_EDITOR_CSS }
        div {
            class,
            if props.line_numbers {
                div { class: "dxc-editor-gutter", aria_hidden: "true",
                    for index in 0..line_count {
                        div { class: "dxc-editor-gutter-line", "{index + 1}" }
                    }
                }
            }
            div { class: "dxc-editor-viewport",
                div { class: "dxc-editor-highlight", aria_hidden: "true",
                    for line in lines {
                        div { class: "dxc-editor-line",
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
                    key: "{input_version}",
                    class: "dxc-editor-input",
                    contenteditable,
                    spellcheck: props.spellcheck,
                    role: "textbox",
                    "aria-label": props.aria_label,
                    "aria-multiline": "true",
                    "aria-readonly": readonly,
                    "data-placeholder": props.placeholder,
                    oninput: move |event| {
                        let value = event.value();
                        input_sync.borrow_mut().last_local_value = value.clone();
                        props.oninput.call(value);
                    },
                    "{input_value}"
                }
            }
        }
    }
}

#[derive(Debug)]
struct InputSyncState {
    rendered_value: String,
    last_local_value: String,
    version: usize,
}

fn synced_input_value(input_sync: &Rc<RefCell<InputSyncState>>, value: &str) -> (String, usize) {
    let mut state = input_sync.borrow_mut();

    if value != state.last_local_value {
        state.last_local_value = value.to_string();
        state.rendered_value = value.to_string();
        state.version += 1;
    }

    (state.rendered_value.clone(), state.version)
}

fn editor_class(theme: Theme, line_numbers: bool, extra_class: &str) -> String {
    let mut class = format!("dxc-editor {}", theme.class());
    if !line_numbers {
        class.push_str(" dxc-editor-no-gutter");
    }
    if !extra_class.is_empty() {
        class.push(' ');
        class.push_str(extra_class);
    }
    class
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn editor_class_includes_theme_and_extra_class() {
        assert_eq!(
            editor_class(Theme::TOKYO_NIGHT, true, "is-compact"),
            "dxc-editor dxc-tokyo-night is-compact",
        );
    }

    #[test]
    fn editor_class_can_hide_gutter() {
        assert_eq!(
            editor_class(Theme::TOKYO_NIGHT, false, ""),
            "dxc-editor dxc-tokyo-night dxc-editor-no-gutter",
        );
    }

    #[test]
    fn lines_preserve_trailing_empty_line() {
        let lines = editor_lines("let x = 1;\n", &[]);
        assert_eq!(lines.len(), 2);
        assert_eq!(
            lines[0],
            vec![Segment {
                text: "let x = 1;",
                tag: None,
            }]
        );
        assert!(lines[1].is_empty());
    }

    #[test]
    fn local_input_echo_does_not_rewrite_editable_text() {
        let input_sync = Rc::new(RefCell::new(InputSyncState {
            rendered_value: "abc".to_string(),
            last_local_value: "abc".to_string(),
            version: 0,
        }));

        input_sync.borrow_mut().last_local_value = "abxc".to_string();

        assert_eq!(
            synced_input_value(&input_sync, "abxc"),
            ("abc".to_string(), 0)
        );
    }

    #[test]
    fn external_value_change_resyncs_editable_text() {
        let input_sync = Rc::new(RefCell::new(InputSyncState {
            rendered_value: "abc".to_string(),
            last_local_value: "abxc".to_string(),
            version: 0,
        }));

        assert_eq!(
            synced_input_value(&input_sync, "xyz"),
            ("xyz".to_string(), 1)
        );
    }

    #[test]
    fn external_reset_to_rendered_value_still_forces_remount() {
        let input_sync = Rc::new(RefCell::new(InputSyncState {
            rendered_value: "abc".to_string(),
            last_local_value: "abxc".to_string(),
            version: 0,
        }));

        assert_eq!(
            synced_input_value(&input_sync, "abc"),
            ("abc".to_string(), 1)
        );
    }
}
