#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

use dioxus::prelude::*;
use dioxus_code::CodeTheme;
pub use dioxus_code::Language;
#[cfg(test)]
use dioxus_code::Theme;
use dioxus_code::advanced::{CodeThemeStyles, IncrementalHighlighter, TokenSpan};
#[cfg(test)]
use dioxus_code::advanced::{HighlightSegment, HighlightedSource};
use std::{cell::RefCell, rc::Rc};

mod edit_capture;

/// Base stylesheet injected by [`CodeEditor`].
///
/// ```rust
/// use dioxus_code_editor::CODE_EDITOR_CSS;
/// let _href = CODE_EDITOR_CSS;
/// ```
pub const CODE_EDITOR_CSS: Asset = asset!("/assets/dioxus-code-editor.css");

/// Props for [`CodeEditor`].
///
/// ```rust,no_run
/// use dioxus_code::{CodeTheme, Theme};
/// use dioxus_code_editor::{CodeEditorProps, Language};
/// let _props = CodeEditorProps::builder()
///     .value("fn main() {}")
///     .language(Some(Language::Rust))
///     .theme(CodeTheme::fixed(Theme::TOKYO_NIGHT))
///     .build();
/// ```
#[derive(Props, Clone, PartialEq)]
pub struct CodeEditorProps {
    /// The current editor contents.
    #[props(into)]
    pub value: String,
    /// Tree-sitter grammar used for syntax highlighting.
    ///
    /// Pass a [`Language`] variant directly, or use [`Language::from_slug`] for
    /// custom Arborium slugs. When unset, the grammar is inferred from the source.
    #[props(into, default)]
    pub language: Option<Language>,
    /// Syntax theme selection shared with [`dioxus-code`].
    ///
    /// [`dioxus-code`]: https://docs.rs/dioxus-code/latest/dioxus_code/
    #[props(default, into)]
    pub theme: CodeTheme,
    /// Show a gutter with one-based line numbers.
    #[props(default = true)]
    pub line_numbers: bool,
    /// Disable editing while preserving syntax highlighting and text selection.
    #[props(default = false)]
    pub read_only: bool,
    /// Forward spellcheck to the textarea input layer.
    #[props(default = false)]
    pub spellcheck: bool,
    /// Accessible label for the editor textbox.
    #[props(into, default = "Code editor")]
    pub aria_label: String,
    /// Placeholder shown only while [`CodeEditorProps::value`] is empty.
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
/// The component is controlled by [`CodeEditorProps::value`]; update that value
/// from [`CodeEditorProps::oninput`] to keep the highlight layer and editable
/// layer in sync.
///
/// ```rust
/// use dioxus::prelude::*;
/// use dioxus_code::Theme;
/// use dioxus_code_editor::{CodeEditor, Language};
///
/// fn _example() -> Element {
///     let mut source = use_signal(String::new);
///     rsx! {
///         CodeEditor {
///             value: source(),
///             language: Language::Rust,
///             theme: Theme::TOKYO_NIGHT,
///             oninput: move |value| source.set(value),
///         }
///     }
/// }
/// ```
#[component]
pub fn CodeEditor(props: CodeEditorProps) -> Element {
    let highlighter = use_hook(|| Rc::new(RefCell::new(IncrementalHighlighter::new())));
    let edit_tracker = use_hook(|| {
        Rc::new(RefCell::new(edit_capture::InputEditTracker::new(
            props.value.clone(),
        )))
    });

    let language = props.language.as_ref().map(Language::slug);
    let edit = edit_tracker.borrow_mut().take_for_render(&props.value);
    let source = highlighter
        .borrow_mut()
        .highlight(&props.value, edit, language);
    let lines = source.lines();
    let line_count = lines.len();
    let class = editor_class(props.theme, props.line_numbers, &props.class);
    let textarea_value = props.value.clone();
    let readonly = props.read_only.then_some("true");
    let input_attributes = edit_capture::use_input_edit_attributes(edit_tracker.clone(), {
        let oninput = props.oninput;
        move |value| oninput.call(value)
    });

    rsx! {
        CodeThemeStyles { theme: props.theme }
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
                                if let Some(tag) = segment.tag() {
                                    TokenSpan {
                                        text: segment.text(),
                                        tag,
                                    }
                                } else {
                                    span { "{segment.text()}" }
                                }
                            }
                        }
                    }
                }
                textarea {
                    class: "dxc-editor-input",
                    readonly: props.read_only,
                    spellcheck: props.spellcheck,
                    role: "textbox",
                    "aria-label": props.aria_label,
                    "aria-multiline": "true",
                    "aria-readonly": readonly,
                    placeholder: props.placeholder,
                    value: textarea_value,
                    wrap: "off",
                    ..input_attributes,
                }
            }
        }
    }
}

fn editor_class(theme: impl Into<CodeTheme>, line_numbers: bool, extra_class: &str) -> String {
    let mut class = format!("dxc-editor {}", theme.into().classes());
    if !line_numbers {
        class.push_str(" dxc-editor-no-gutter");
    }
    if !extra_class.is_empty() {
        class.push(' ');
        class.push_str(extra_class);
    }
    class
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
    fn editor_class_can_use_system_theme() {
        assert_eq!(
            editor_class(
                CodeTheme::system(Theme::GITHUB_LIGHT, Theme::TOKYO_NIGHT),
                true,
                "",
            ),
            "dxc-editor dxc-system dxc-system-light-github-light dxc-system-dark-tokyo-night",
        );
    }

    #[test]
    fn lines_preserve_trailing_empty_line() {
        let source = HighlightedSource::from_static_parts("let x = 1;\n", "rust", &[]);
        let lines = source.lines();
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0], vec![HighlightSegment::new("let x = 1;", None)]);
        assert!(lines[1].is_empty());
    }
}
