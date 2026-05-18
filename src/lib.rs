#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

extern crate self as dioxus_code;

use dioxus::prelude::*;
#[cfg(feature = "runtime")]
use std::collections::HashMap;

mod language;
pub use language::Language;

const CODE_CSS: Asset = asset!("/assets/dioxus-code.css");

#[cfg(feature = "macro")]
#[cfg_attr(docsrs, doc(cfg(feature = "macro")))]
pub use dioxus_code_macro::{code, code_str};

/// Compile-time options for the [`code!`] and [`code_str!`] macros.
///
/// Both macros read this builder syntactically; pass
/// [`CodeOptions::builder`] with [`CodeOptions::with_language`] to override the
/// language that would otherwise be inferred from the file extension. For
/// [`code_str!`] the language is required since there is no extension to
/// infer from.
///
/// ```rust
/// use dioxus_code::{CodeOptions, Language, code};
///
/// let _source = code!(
///     "/snippets/demo.rs",
///     CodeOptions::builder().with_language(Language::Rust)
/// );
/// ```
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct CodeOptions {
    language: Option<Language>,
}

impl CodeOptions {
    /// Create default code options.
    ///
    /// ```rust
    /// use dioxus_code::CodeOptions;
    /// let _opts = CodeOptions::new();
    /// ```
    pub const fn new() -> Self {
        Self { language: None }
    }

    /// Create default code options.
    ///
    /// Alias for [`Self::new`], matching builder-style asset APIs.
    ///
    /// ```rust
    /// use dioxus_code::{CodeOptions, Language};
    /// let _opts = CodeOptions::builder().with_language(Language::Rust);
    /// ```
    pub const fn builder() -> Self {
        Self::new()
    }

    /// Set the language explicitly.
    ///
    /// Pass a [`Language`] variant directly, `Some(Language::...)`, or `None`.
    ///
    /// ```rust
    /// use dioxus_code::{CodeOptions, Language};
    /// let _opts = CodeOptions::new().with_language(Language::Rust);
    /// ```
    pub fn with_language(mut self, language: impl Into<Option<Language>>) -> Self {
        self.language = language.into();
        self
    }

    /// The explicit language, if one was configured.
    pub const fn language(self) -> Option<Language> {
        self.language
    }
}

/// A syntax-highlighting theme.
///
/// Themes are exposed as associated constants on [`Theme`] (for example
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
pub use advanced::{HighlightError, HighlightQueryErrorKind};

/// Source text to highlight at runtime.
///
/// Available with the `runtime` feature. Build one with [`SourceCode::new`],
/// then pass it to [`Code()`].
///
/// ```rust
/// use dioxus_code::{Language, SourceCode};
/// let _src = SourceCode::new(Language::Rust, "fn main() {}");
/// ```
#[cfg(feature = "runtime")]
#[cfg_attr(docsrs, doc(cfg(feature = "runtime")))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceCode {
    source: String,
    language: Language,
}

/// Source-first builder for [`SourceCode`].
#[cfg(feature = "runtime")]
#[cfg_attr(docsrs, doc(cfg(feature = "runtime")))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceCodeBuilder {
    source: String,
}

#[cfg(feature = "runtime")]
#[cfg_attr(docsrs, doc(cfg(feature = "runtime")))]
impl SourceCode {
    /// Wrap a raw source string with an explicit language.
    ///
    /// ```rust
    /// use dioxus_code::{Language, SourceCode};
    /// let _src = SourceCode::new(Language::Rust, "fn main() {}");
    /// ```
    pub fn new(language: Language, source: impl ToString) -> Self {
        Self {
            source: source.to_string(),
            language,
        }
    }

    /// Start a source-first builder.
    pub fn builder(source: impl ToString) -> SourceCodeBuilder {
        SourceCodeBuilder {
            source: source.to_string(),
        }
    }

    /// Replace the language used to highlight this source.
    ///
    /// To set the language from a runtime slug, use [`Language::from_slug`]
    /// and pass the resulting variant.
    ///
    /// ```rust
    /// use dioxus_code::{Language, SourceCode};
    /// let _src = SourceCode::new(Language::Rust, "fn main() {}").with_language(Language::Rust);
    /// ```
    pub fn with_language(mut self, language: Language) -> Self {
        self.language = language;
        self
    }

    /// Highlight this source, returning typed errors when runtime highlighting fails.
    ///
    /// Use `Into<HighlightedSource>` for the lossy rendering path that discards
    /// the error and renders plaintext.
    pub fn highlight(self) -> Result<advanced::HighlightedSource, HighlightError> {
        advanced::Buffer::new(self.language, self.source).map(|buffer| buffer.highlighted())
    }

    fn highlight_or_plaintext(self) -> advanced::HighlightedSource {
        let language = self.language;
        let source = self.source.clone();
        match self.highlight() {
            Ok(source) => source,
            Err(_) => advanced::HighlightedSource::plaintext(source, language),
        }
    }
}

#[cfg(feature = "runtime")]
#[cfg_attr(docsrs, doc(cfg(feature = "runtime")))]
impl SourceCodeBuilder {
    /// Finish the builder with an explicit language.
    pub fn with_language(self, language: Language) -> SourceCode {
        SourceCode::new(language, self.source)
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
        code.highlight_or_plaintext()
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
/// Pair the [`code!`] macro for compile-time parsing, or [`SourceCode`] for
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
    let language = source.language().slug();

    rsx! {
        advanced::CodeThemeStyles { theme: props.theme }
        document::Stylesheet { href: CODE_CSS }
        pre {
            class,
            "data-language": language,
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
                Language::Rust,
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
        let source =
            advanced::HighlightedSource::from_static_parts("let x = 1;\n", Language::Rust, &[]);
        let lines = source.lines();
        assert_eq!(lines.len(), 2);
        assert_eq!(
            lines[0],
            vec![advanced::HighlightSegment::new("let x = 1;", None)]
        );
        assert!(lines[1].is_empty());
    }

    #[test]
    fn code_options_accepts_language_options() {
        assert_eq!(
            CodeOptions::builder()
                .with_language(Language::Rust)
                .language(),
            Some(Language::Rust),
        );
        assert_eq!(
            CodeOptions::builder()
                .with_language(Some(Language::Rust))
                .language(),
            Some(Language::Rust),
        );
        assert_eq!(CodeOptions::builder().with_language(None).language(), None);
    }

    #[cfg(feature = "runtime")]
    #[test]
    fn runtime_source_code_highlights() {
        let tree: advanced::HighlightedSource =
            SourceCode::new(Language::Rust, "fn main() {}").into();
        assert_eq!(tree.language(), Language::Rust);
        assert!(tree.spans().iter().any(|span| {
            span.tag() == "k" && &tree.source()[span.start() as usize..span.end() as usize] == "fn"
        }));
    }

    #[cfg(feature = "macro")]
    #[test]
    fn code_str_macro_highlights_inline_source() {
        const TREE: advanced::HighlightedSource = code_str!(
            "fn main() {}",
            CodeOptions::builder().with_language(Language::Rust)
        );
        assert_eq!(TREE.language(), Language::Rust);
        assert_eq!(TREE.source(), "fn main() {}");
        assert!(TREE.spans().iter().any(|span| {
            span.tag() == "k" && &TREE.source()[span.start() as usize..span.end() as usize] == "fn"
        }));
    }

    #[cfg(all(feature = "macro", feature = "detection"))]
    #[test]
    fn code_str_macro_detects_inline_source() {
        const TREE: advanced::HighlightedSource = code_str!("fn main() { println!(\"hi\"); }");

        assert_eq!(TREE.language(), Language::Rust);
        assert_eq!(TREE.source(), "fn main() { println!(\"hi\"); }");
        assert!(TREE.spans().iter().any(|span| {
            span.tag() == "k" && &TREE.source()[span.start() as usize..span.end() as usize] == "fn"
        }));
    }
}
