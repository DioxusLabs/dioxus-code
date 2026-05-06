#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

extern crate self as dioxus_code;

#[cfg(feature = "runtime")]
use arborium::advanced::Span;
#[cfg(feature = "runtime")]
use arborium_theme::tag_for_capture;
use dioxus::prelude::*;
#[cfg(feature = "runtime")]
use std::collections::HashMap;

/// Base stylesheet for [`Code()`].
///
/// This contains layout styles only; syntax colors live in the generated theme
/// assets and the shared generated theme rule asset.
///
/// ```rust
/// use dioxus_code::CODE_CSS;
/// let _href = CODE_CSS;
/// ```
pub const CODE_CSS: Asset = asset!("/assets/dioxus-code.css");

#[cfg(feature = "macro")]
#[cfg_attr(docsrs, doc(cfg(feature = "macro")))]
pub use dioxus_code_macro::code;

/// Options for the `code!` macro.
///
/// The `code!` macro reads this builder syntax at compile time.
///
/// ```rust
/// use dioxus_code::{CodeOptions, code};
///
/// let _source = code!(
///     "/snippets/Containerfile",
///     CodeOptions::builder().with_language("dockerfile")
/// );
/// ```
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct CodeOptions {
    _private: (),
}

impl CodeOptions {
    /// Create default macro options.
    ///
    /// ```rust
    /// use dioxus_code::CodeOptions;
    /// let _opts = CodeOptions::new();
    /// ```
    pub const fn new() -> Self {
        Self { _private: () }
    }

    /// Create default macro options.
    ///
    /// Alias for [`CodeOptions::new`], matching builder-style asset APIs.
    ///
    /// ```rust
    /// use dioxus_code::CodeOptions;
    /// let _opts = CodeOptions::builder().with_language("rust");
    /// ```
    pub const fn builder() -> Self {
        Self::new()
    }

    /// Set the language explicitly.
    ///
    /// The macro parses this call during expansion.
    ///
    /// ```rust
    /// use dioxus_code::CodeOptions;
    /// let _opts = CodeOptions::new().with_language("rust");
    /// ```
    pub const fn with_language(self, _language: &'static str) -> Self {
        self
    }
}

/// A syntax-highlighting theme.
///
/// Themes are exposed as associated constants on `Theme` (for example
/// [`Theme::TOKYO_NIGHT`]) and ship as scoped CSS so multiple themes can
/// coexist on the same page without leaking styles.
///
/// ```rust
/// use dioxus_code::Theme;
/// let _theme = Theme::TOKYO_NIGHT;
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Theme {
    stylesheet: ThemeStylesheet,
    system_light: ThemeStylesheet,
    system_dark: ThemeStylesheet,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct ThemeStylesheet {
    class: &'static str,
    asset: Asset,
}

impl Theme {
    const fn stylesheet(self) -> ThemeStylesheet {
        self.stylesheet
    }

    const fn system_light(self) -> ThemeStylesheet {
        self.system_light
    }

    const fn system_dark(self) -> ThemeStylesheet {
        self.system_dark
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::RUSTDOC_AYU
    }
}

/// Syntax theme selection for [`Code()`].
///
/// ```rust
/// use dioxus_code::{CodeTheme, Theme};
/// let _theme = CodeTheme::fixed(Theme::TOKYO_NIGHT);
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CodeTheme {
    selection: CodeThemeSelection,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum CodeThemeChoice<T> {
    Fixed(T),
    System { light: T, dark: T },
}

type CodeThemeSelection = CodeThemeChoice<Theme>;
type CodeThemeStylesheets = CodeThemeChoice<ThemeStylesheet>;

impl CodeTheme {
    /// Create a fixed theme selection.
    ///
    /// ```rust
    /// use dioxus_code::{CodeTheme, Theme};
    /// let _theme = CodeTheme::fixed(Theme::TOKYO_NIGHT);
    /// ```
    pub const fn fixed(theme: Theme) -> Self {
        Self {
            selection: CodeThemeSelection::Fixed(theme),
        }
    }

    /// Create a CSS-only system theme pair.
    ///
    /// ```rust
    /// use dioxus_code::{CodeTheme, Theme};
    /// let _theme = CodeTheme::system(Theme::GITHUB_LIGHT, Theme::TOKYO_NIGHT);
    /// ```
    pub const fn system(light: Theme, dark: Theme) -> Self {
        Self {
            selection: CodeThemeSelection::System { light, dark },
        }
    }

    /// CSS classes to apply to a code container using this theme selection.
    ///
    /// ```rust
    /// use dioxus_code::{CodeTheme, Theme};
    /// let classes = CodeTheme::fixed(Theme::TOKYO_NIGHT).classes();
    /// assert!(classes.contains("dxc-tokyo-night"));
    /// ```
    pub fn classes(self) -> String {
        match self.stylesheets() {
            CodeThemeStylesheets::Fixed(stylesheet) => stylesheet.class.to_string(),
            CodeThemeStylesheets::System { light, dark } => {
                format!("dxc-system {} {}", light.class, dark.class)
            }
        }
    }

    const fn stylesheets(self) -> CodeThemeStylesheets {
        match self.selection {
            CodeThemeSelection::Fixed(theme) => CodeThemeStylesheets::Fixed(theme.stylesheet()),
            CodeThemeSelection::System { light, dark } => CodeThemeStylesheets::System {
                light: light.system_light(),
                dark: dark.system_dark(),
            },
        }
    }
}

impl Default for CodeTheme {
    fn default() -> Self {
        Self::fixed(Theme::default())
    }
}

impl From<Theme> for CodeTheme {
    fn from(theme: Theme) -> Self {
        Self::fixed(theme)
    }
}

include!(concat!(env!("OUT_DIR"), "/theme_assets.rs"));

pub mod advanced;

/// Source text to highlight at runtime.
///
/// Available with the `runtime` feature. Build one with [`SourceCode::new`],
/// optionally annotate it with a language or filename, then pass it to
/// [`Code()`].
///
/// ```rust
/// use dioxus_code::SourceCode;
/// let _src = SourceCode::new("fn main() {}").with_language("rust");
/// ```
#[cfg(feature = "runtime")]
#[cfg_attr(docsrs, doc(cfg(feature = "runtime")))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceCode {
    source: String,
    language: Option<String>,
    filename: Option<String>,
}

#[cfg(feature = "runtime")]
#[cfg_attr(docsrs, doc(cfg(feature = "runtime")))]
impl SourceCode {
    /// Wrap a raw source string with no language hint.
    ///
    /// ```rust
    /// use dioxus_code::SourceCode;
    /// let _src = SourceCode::new("fn main() {}");
    /// ```
    pub fn new(source: impl Into<String>) -> Self {
        Self {
            source: source.into(),
            language: None,
            filename: None,
        }
    }

    /// Set the language explicitly.
    ///
    /// Accepts an Arborium language slug such as `"rust"`.
    ///
    /// ```rust
    /// use dioxus_code::SourceCode;
    /// let _src = SourceCode::new("fn main() {}").with_language("rust");
    /// ```
    pub fn with_language(mut self, language: impl Into<String>) -> Self {
        self.language = Some(language.into());
        self
    }

    /// Set a filename used to infer the language when none is set explicitly.
    ///
    /// ```rust
    /// use dioxus_code::SourceCode;
    /// let _src = SourceCode::new("fn main() {}").with_filename("main.rs");
    /// ```
    pub fn with_filename(mut self, filename: impl Into<String>) -> Self {
        self.filename = Some(filename.into());
        self
    }

    /// Set a filename used to infer the language when none is set explicitly.
    ///
    /// ```rust
    /// # #[allow(deprecated)] {
    /// use dioxus_code::SourceCode;
    /// let _src = SourceCode::new("fn main() {}").with_name("main.rs");
    /// # }
    /// ```
    #[deprecated = "use SourceCode::with_filename instead"]
    pub fn with_name(self, name: impl Into<String>) -> Self {
        self.with_filename(name)
    }

    fn highlight(self) -> advanced::HighlightedSource {
        let mut highlighter = advanced::IncrementalHighlighter::new();
        highlighter.highlight(
            &self.source,
            None,
            self.language.as_deref(),
            self.filename.as_deref(),
        )
    }
}

#[cfg(feature = "runtime")]
pub(crate) struct RawHighlightSpan {
    pub(crate) start: u32,
    pub(crate) end: u32,
    pub(crate) tag: Option<&'static str>,
    pub(crate) pattern_index: u32,
}

#[cfg(feature = "runtime")]
impl From<Span> for RawHighlightSpan {
    fn from(span: Span) -> Self {
        Self {
            start: span.start,
            end: span.end,
            tag: tag_for_capture(&span.capture),
            pattern_index: span.pattern_index,
        }
    }
}

#[cfg(feature = "runtime")]
pub(crate) fn normalize_spans(
    spans: impl IntoIterator<Item = RawHighlightSpan>,
) -> Vec<advanced::HighlightSpan> {
    let mut deduped: HashMap<(u32, u32), RawHighlightSpan> = HashMap::new();

    for span in spans.into_iter() {
        let key = (span.start, span.end);
        if let Some(existing) = deduped.get(&key) {
            let should_replace = match (span.tag.is_some(), existing.tag.is_some()) {
                (true, false) => true,
                (false, true) => false,
                _ => span.pattern_index >= existing.pattern_index,
            };
            if should_replace {
                deduped.insert(key, span);
            }
        } else {
            deduped.insert(key, span);
        }
    }

    let mut spans: Vec<_> = deduped
        .into_values()
        .filter_map(|span| {
            Some(advanced::HighlightSpan::new(
                span.start..span.end,
                span.tag?,
            ))
        })
        .collect();

    spans.sort_by_key(|span| (span.start(), span.end()));

    let mut coalesced: Vec<advanced::HighlightSpan> = Vec::with_capacity(spans.len());
    for span in spans {
        if let Some(last) = coalesced.last_mut()
            && span.tag() == last.tag()
            && span.start() <= last.end()
        {
            last.set_end(last.end().max(span.end()));
            continue;
        }
        coalesced.push(span);
    }

    coalesced
}

#[cfg(feature = "runtime")]
#[cfg_attr(docsrs, doc(cfg(feature = "runtime")))]
impl From<SourceCode> for advanced::HighlightedSource {
    fn from(code: SourceCode) -> Self {
        code.highlight()
    }
}

/// Props for [`Code()`].
///
/// ```rust
/// use dioxus_code::{CodeProps, Theme, code};
/// let _props = CodeProps {
///     src: code!("/snippets/demo.rs"),
///     theme: Theme::TOKYO_NIGHT.into(),
/// };
/// ```
#[derive(Props, Clone, PartialEq)]
pub struct CodeProps {
    /// Source to render.
    #[props(into)]
    pub src: advanced::HighlightedSource,
    /// Syntax theme. Defaults to [`Theme::RUSTDOC_AYU`].
    #[props(default, into)]
    pub theme: CodeTheme,
}

/// Render syntax-highlighted source code.
///
/// Pair the `code!` macro for compile-time parsing, or `SourceCode` for
/// runtime parsing with the `runtime` feature. The component injects its own
/// stylesheet plus the selected theme's stylesheet.
///
/// ```rust
/// use dioxus::prelude::*;
/// use dioxus_code::{Code, Theme, code};
///
/// fn _example() -> Element {
///     rsx! {
///         Code { src: code!("/snippets/demo.rs"), theme: Theme::TOKYO_NIGHT }
///     }
/// }
/// ```
#[component]
pub fn Code(props: CodeProps) -> Element {
    let source = &props.src;
    let segments = source.trimmed_segments();
    let class = format!("dxc {}", props.theme.classes());
    let language = source.language().unwrap_or("text");
    let error = source.error();

    rsx! {
        advanced::CodeThemeStyles { theme: props.theme }
        document::Stylesheet { href: CODE_CSS }
        pre {
            class,
            "data-language": language,
            "data-error": error,
            code {
                for segment in segments {
                    if let Some(tag) = segment.tag() {
                        advanced::TokenSpan {
                            text: segment.text(),
                            tag,
                        }
                    } else {
                        span {
                            "{segment.text()}"
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn system_theme_classes_include_scoped_slots() {
        assert_eq!(
            CodeTheme::system(Theme::GITHUB_LIGHT, Theme::TOKYO_NIGHT).classes(),
            "dxc-system dxc-system-light-github-light dxc-system-dark-tokyo-night",
        );
    }

    #[test]
    fn plaintext_is_escaped() {
        assert_eq!(
            advanced::HighlightedSource::from_static_parts(
                "<script>alert(1)</script>",
                "text",
                &[]
            )
            .segments(),
            vec![advanced::HighlightSegment::new(
                "<script>alert(1)</script>",
                None,
            )]
        );
    }

    #[test]
    fn highlighted_lines_preserve_trailing_empty_line() {
        let source = advanced::HighlightedSource::from_static_parts("let x = 1;\n", "rust", &[]);
        let lines = source.lines();
        assert_eq!(lines.len(), 2);
        assert_eq!(
            lines[0],
            vec![advanced::HighlightSegment::new("let x = 1;", None)]
        );
        assert!(lines[1].is_empty());
    }

    #[cfg(feature = "runtime")]
    #[test]
    fn runtime_filename_detection_highlights() {
        let tree: advanced::HighlightedSource = SourceCode::new("fn main() {}")
            .with_filename("main.rs")
            .into();
        assert_eq!(tree.language(), Some("rust"));
        assert!(tree.spans().iter().any(|span| {
            span.tag() == "k" && &tree.source()[span.start() as usize..span.end() as usize] == "fn"
        }));
    }

    #[cfg(feature = "runtime")]
    #[test]
    fn runtime_language_string_highlights() {
        let tree: advanced::HighlightedSource =
            SourceCode::new("fn main() {}").with_language("rust").into();
        assert_eq!(tree.language(), Some("rust"));
        assert!(tree.spans().iter().any(|span| {
            span.tag() == "k" && &tree.source()[span.start() as usize..span.end() as usize] == "fn"
        }));
    }

    #[cfg(feature = "runtime")]
    #[test]
    fn runtime_raw_string_uses_arborium_detection_fallback() {
        let tree: advanced::HighlightedSource = SourceCode::new("fn main() {}").into();
        assert_eq!(tree.language(), None);
        assert_eq!(tree.error(), Some("could not detect language"));
    }
}
