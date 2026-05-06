#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

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
pub const CODE_CSS: Asset = asset!("/assets/dioxus-code.css");

#[cfg(feature = "macro")]
pub use dioxus_code_macro::code;

/// Options for the `code!` macro.
///
/// The `code!` macro reads this builder syntax at compile time.
///
/// ```rust,ignore
/// use dioxus_code::{CodeOptions, code};
///
/// let source = code!(
///     "/snippets/Containerfile",
///     CodeOptions::new().with_language("dockerfile")
/// );
/// ```
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct CodeOptions {
    _private: (),
}

impl CodeOptions {
    /// Create default macro options.
    pub const fn new() -> Self {
        Self { _private: () }
    }

    /// Create default macro options.
    ///
    /// Alias for [`CodeOptions::new`], matching builder-style asset APIs.
    pub const fn builder() -> Self {
        Self::new()
    }

    /// Create default macro options.
    ///
    /// Equivalent to [`CodeOptions::new`].
    pub const fn default() -> Self {
        Self::new()
    }

    /// Set the language explicitly.
    ///
    /// The macro parses this call during expansion.
    pub const fn with_language(self, _language: &'static str) -> Self {
        self
    }
}

/// A syntax-highlighting theme.
///
/// Themes are exposed as associated constants on `Theme` (for example
/// [`Theme::TOKYO_NIGHT`]) and ship as scoped CSS so multiple themes can
/// coexist on the same page without leaking styles.
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
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CodeTheme {
    selection: CodeThemeSelection,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum CodeThemeSelection {
    Fixed(Theme),
    System { light: Theme, dark: Theme },
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum CodeThemeStylesheets {
    Fixed(ThemeStylesheet),
    System {
        light: ThemeStylesheet,
        dark: ThemeStylesheet,
    },
}

impl CodeTheme {
    /// Create a fixed theme selection.
    pub const fn fixed(theme: Theme) -> Self {
        Self {
            selection: CodeThemeSelection::Fixed(theme),
        }
    }

    /// Create a CSS-only system theme pair.
    pub const fn system(light: Theme, dark: Theme) -> Self {
        Self {
            selection: CodeThemeSelection::System { light, dark },
        }
    }

    /// CSS classes to apply to a code container using this theme selection.
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
#[cfg(feature = "runtime")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceCode {
    source: String,
    language: Option<String>,
    name: Option<String>,
}

#[cfg(feature = "runtime")]
impl SourceCode {
    /// Wrap a raw source string with no language hint.
    pub fn new(source: impl Into<String>) -> Self {
        Self {
            source: source.into(),
            language: None,
            name: None,
        }
    }

    /// Set the language explicitly.
    ///
    /// Accepts an Arborium language slug such as `"rust"`.
    pub fn with_language(mut self, language: impl Into<String>) -> Self {
        self.language = Some(language.into());
        self
    }

    /// Set a filename used to infer the language when none is set explicitly.
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    fn highlight(self) -> advanced::HighlightedSource {
        let language = self
            .language
            .or_else(|| {
                self.name
                    .as_deref()
                    .and_then(arborium::detect_language)
                    .map(str::to_string)
            })
            .or_else(|| arborium::detect_language(&self.source).map(str::to_string));

        let Some(language) = language else {
            return advanced::HighlightedSource::plaintext(
                self.source,
                "could not detect language",
            );
        };

        let mut highlighter = arborium::Highlighter::new();
        match highlighter.highlight_spans(&language, &self.source) {
            Ok(spans) => advanced::HighlightedSource::from_owned_parts(
                self.source,
                Some(language),
                normalize_spans(spans),
                None,
            ),
            Err(error) => advanced::HighlightedSource::plaintext(self.source, error.to_string()),
        }
    }
}

#[cfg(feature = "runtime")]
struct RawHighlightSpan {
    start: u32,
    end: u32,
    tag: Option<&'static str>,
    pattern_index: u32,
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
fn normalize_spans(spans: impl IntoIterator<Item = Span>) -> Vec<advanced::HighlightSpan> {
    let mut deduped: HashMap<(u32, u32), RawHighlightSpan> = HashMap::new();

    for span in spans.into_iter().map(RawHighlightSpan::from) {
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
                span.start, span.end, span.tag?,
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
impl From<SourceCode> for advanced::HighlightedSource {
    fn from(code: SourceCode) -> Self {
        code.highlight()
    }
}

/// Props for [`Code()`].
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
                            text: segment.text().to_string(),
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
    fn runtime_name_detection_highlights() {
        let tree: advanced::HighlightedSource =
            SourceCode::new("fn main() {}").with_name("main.rs").into();
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
