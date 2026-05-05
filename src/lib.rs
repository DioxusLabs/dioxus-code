use arborium::HtmlFormat;
use arborium::advanced::{Span, html_escape, spans_to_html};
use dioxus::prelude::*;

const STYLE: Asset = asset!("/assets/dioxus-code.css");

#[cfg(feature = "macro")]
pub use dioxus_code_macro::code;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Theme {
    name: &'static str,
    class: &'static str,
    asset: Asset,
}

impl Theme {
    pub const fn name(self) -> &'static str {
        self.name
    }

    pub const fn class(self) -> &'static str {
        self.class
    }

    pub const fn asset(self) -> Asset {
        self.asset
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::RUSTDOC_AYU
    }
}

include!(concat!(env!("OUT_DIR"), "/theme_assets.rs"));

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CodeTree {
    source: String,
    language: Option<String>,
    spans: Vec<HighlightSpan>,
    error: Option<String>,
}

impl CodeTree {
    pub fn from_static_parts(
        source: &'static str,
        language: &'static str,
        spans: &'static [StaticSpan],
    ) -> Self {
        Self {
            source: source.to_string(),
            language: Some(language.to_string()),
            spans: spans.iter().copied().map(HighlightSpan::from).collect(),
            error: None,
        }
    }

    pub fn plaintext(source: impl Into<String>, error: impl Into<String>) -> Self {
        Self {
            source: source.into(),
            language: None,
            spans: Vec::new(),
            error: Some(error.into()),
        }
    }

    fn html(&self) -> String {
        if self.spans.is_empty() {
            return html_escape(self.source.trim_end_matches('\n'));
        }

        let spans = self
            .spans
            .iter()
            .map(|span| Span {
                start: span.start,
                end: span.end,
                capture: span.capture.to_string(),
                pattern_index: span.pattern_index,
            })
            .collect();

        spans_to_html(&self.source, spans, &HtmlFormat::CustomElements)
    }

    fn language(&self) -> Option<&str> {
        self.language.as_deref()
    }

    fn error(&self) -> Option<&str> {
        self.error.as_deref()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StaticSpan {
    pub start: u32,
    pub end: u32,
    pub capture: &'static str,
    pub pattern_index: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct HighlightSpan {
    start: u32,
    end: u32,
    capture: String,
    pattern_index: u32,
}

impl From<StaticSpan> for HighlightSpan {
    fn from(span: StaticSpan) -> Self {
        Self {
            start: span.start,
            end: span.end,
            capture: span.capture.to_string(),
            pattern_index: span.pattern_index,
        }
    }
}

impl From<Span> for HighlightSpan {
    fn from(span: Span) -> Self {
        Self {
            start: span.start,
            end: span.end,
            capture: span.capture,
            pattern_index: span.pattern_index,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeCode {
    source: String,
    language: Option<String>,
    name: Option<String>,
}

impl RuntimeCode {
    pub fn new(source: impl Into<String>) -> Self {
        Self {
            source: source.into(),
            language: None,
            name: None,
        }
    }

    pub fn with_language(mut self, language: impl Into<String>) -> Self {
        self.language = Some(language.into());
        self
    }

    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    #[cfg(feature = "runtime")]
    fn highlight(self) -> CodeTree {
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
            return CodeTree::plaintext(self.source, "could not detect language");
        };

        let mut highlighter = arborium::Highlighter::new();
        match highlighter.highlight_spans(&language, &self.source) {
            Ok(spans) => CodeTree {
                source: self.source,
                language: Some(language),
                spans: spans.into_iter().map(HighlightSpan::from).collect(),
                error: None,
            },
            Err(error) => CodeTree::plaintext(self.source, error.to_string()),
        }
    }

    #[cfg(not(feature = "runtime"))]
    fn highlight(self) -> CodeTree {
        CodeTree::plaintext(
            self.source,
            "runtime parsing requires the dioxus-code runtime feature",
        )
    }
}

pub trait IntoTree {
    fn into_tree(self) -> CodeTree;
}

impl IntoTree for CodeTree {
    fn into_tree(self) -> CodeTree {
        self
    }
}

impl IntoTree for RuntimeCode {
    fn into_tree(self) -> CodeTree {
        self.highlight()
    }
}

#[cfg(feature = "runtime")]
impl IntoTree for &str {
    fn into_tree(self) -> CodeTree {
        RuntimeCode::new(self).highlight()
    }
}

#[cfg(feature = "runtime")]
impl IntoTree for String {
    fn into_tree(self) -> CodeTree {
        RuntimeCode::new(self).highlight()
    }
}

impl From<CodeTree> for CodeSource {
    fn from(tree: CodeTree) -> Self {
        Self(tree)
    }
}

impl From<RuntimeCode> for CodeSource {
    fn from(code: RuntimeCode) -> Self {
        Self(code.into_tree())
    }
}

#[cfg(feature = "runtime")]
impl From<&str> for CodeSource {
    fn from(source: &str) -> Self {
        Self(source.into_tree())
    }
}

#[cfg(feature = "runtime")]
impl From<String> for CodeSource {
    fn from(source: String) -> Self {
        Self(source.into_tree())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CodeSource(CodeTree);

#[derive(Props, Clone, PartialEq)]
pub struct CodeProps {
    #[props(into)]
    pub src: CodeSource,
    #[props(default)]
    pub theme: Theme,
}

#[component]
pub fn Code(props: CodeProps) -> Element {
    let tree = props.src.0;
    let html = tree.html();
    let class = format!("dxc {}", props.theme.class());
    let theme_asset = props.theme.asset();
    let language = tree.language().unwrap_or("text");
    let error = tree.error();

    rsx! {
        document::Stylesheet { href: STYLE }
        document::Stylesheet { href: theme_asset }
        pre {
            class,
            "data-language": language,
            "data-error": error,
            code {
                dangerous_inner_html: "{html}"
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plaintext_is_escaped() {
        let tree = CodeTree::plaintext("<script>alert(1)</script>", "plain");
        assert_eq!(tree.html(), "&lt;script&gt;alert(1)&lt;/script&gt;");
    }

    #[cfg(feature = "runtime")]
    #[test]
    fn runtime_name_detection_highlights() {
        let tree = RuntimeCode::new("fn main() {}")
            .with_name("main.rs")
            .into_tree();
        assert_eq!(tree.language(), Some("rust"));
        assert!(tree.html().contains("<a-k>fn</a-k>"));
    }

    #[cfg(feature = "runtime")]
    #[test]
    fn runtime_raw_string_uses_arborium_detection_fallback() {
        let tree = RuntimeCode::new("fn main() {}").into_tree();
        assert_eq!(tree.language(), None);
        assert_eq!(tree.error(), Some("could not detect language"));
    }
}
