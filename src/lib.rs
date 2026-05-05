#[cfg(feature = "runtime")]
use arborium::advanced::Span;
#[cfg(feature = "runtime")]
use arborium_theme::tag_for_capture;
use dioxus::prelude::*;
#[cfg(feature = "runtime")]
use std::collections::HashMap;

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

    pub fn source(&self) -> &str {
        &self.source
    }

    pub fn language(&self) -> Option<&str> {
        self.language.as_deref()
    }

    pub fn error(&self) -> Option<&str> {
        self.error.as_deref()
    }

    pub fn spans(&self) -> &[HighlightSpan] {
        &self.spans
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StaticSpan {
    pub start: u32,
    pub end: u32,
    pub tag: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HighlightSpan {
    pub start: u32,
    pub end: u32,
    pub tag: &'static str,
}

impl From<StaticSpan> for HighlightSpan {
    fn from(span: StaticSpan) -> Self {
        Self {
            start: span.start,
            end: span.end,
            tag: span.tag,
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
fn normalize_spans(spans: impl IntoIterator<Item = Span>) -> Vec<HighlightSpan> {
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
            Some(HighlightSpan {
                start: span.start,
                end: span.end,
                tag: span.tag?,
            })
        })
        .collect();

    spans.sort_by_key(|span| (span.start, span.end));

    let mut coalesced: Vec<HighlightSpan> = Vec::with_capacity(spans.len());
    for span in spans {
        if let Some(last) = coalesced.last_mut()
            && span.tag == last.tag
            && span.start <= last.end
        {
            last.end = last.end.max(span.end);
            continue;
        }
        coalesced.push(span);
    }

    coalesced
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceCode {
    source: String,
    language: Option<String>,
    name: Option<String>,
}

impl SourceCode {
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
                spans: normalize_spans(spans),
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

impl IntoTree for SourceCode {
    fn into_tree(self) -> CodeTree {
        self.highlight()
    }
}

#[cfg(feature = "runtime")]
impl IntoTree for &str {
    fn into_tree(self) -> CodeTree {
        SourceCode::new(self).highlight()
    }
}

#[cfg(feature = "runtime")]
impl IntoTree for String {
    fn into_tree(self) -> CodeTree {
        SourceCode::new(self).highlight()
    }
}

impl From<CodeTree> for CodeSource {
    fn from(tree: CodeTree) -> Self {
        Self(tree)
    }
}

impl From<SourceCode> for CodeSource {
    fn from(code: SourceCode) -> Self {
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

#[derive(Debug, Clone, PartialEq, Eq)]
struct CodeSegment<'a> {
    text: &'a str,
    tag: Option<&'static str>,
}

fn code_segments<'a>(source: &'a str, spans: &[HighlightSpan]) -> Vec<CodeSegment<'a>> {
    code_segments_inner(source.trim_end_matches('\n'), spans)
}

fn code_segments_inner<'a>(source: &'a str, spans: &[HighlightSpan]) -> Vec<CodeSegment<'a>> {
    if spans.is_empty() {
        return vec![CodeSegment {
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
            push_code_segment(
                &mut segments,
                &source[last_pos..pos],
                stack.last().map(|&i| spans[i].tag),
            );
            last_pos = pos;
        }

        if is_start {
            stack.push(span_index);
        } else if let Some(index) = stack.iter().rposition(|&i| i == span_index) {
            stack.remove(index);
        }
    }

    if last_pos < source.len() {
        push_code_segment(
            &mut segments,
            &source[last_pos..],
            stack.last().map(|&i| spans[i].tag),
        );
    }

    segments
}

fn push_code_segment<'a>(
    segments: &mut Vec<CodeSegment<'a>>,
    text: &'a str,
    tag: Option<&'static str>,
) {
    segments.push(CodeSegment { text, tag });
}

#[derive(Props, Clone, PartialEq)]
pub struct CodeSpanProps {
    pub text: String,
    pub tag: &'static str,
}

#[component]
pub fn CodeSpan(props: CodeSpanProps) -> Element {
    let class = format!("a-{}", props.tag);
    rsx! {
        span {
            class,
            "{props.text}"
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct CodeProps {
    #[props(into)]
    pub src: CodeSource,
    #[props(default)]
    pub theme: Theme,
}

#[component]
pub fn Code(props: CodeProps) -> Element {
    let CodeTree {
        source,
        language,
        spans,
        error,
    } = props.src.0;
    let segments = code_segments(&source, &spans);
    let class = format!("dxc {}", props.theme.class());
    let theme_asset = props.theme.asset();
    let language = language.as_deref().unwrap_or("text");
    let error = error.as_deref();

    rsx! {
        document::Stylesheet { href: STYLE }
        document::Stylesheet { href: theme_asset }
        pre {
            class,
            "data-language": language,
            "data-error": error,
            code {
                for segment in segments {
                    if let Some(tag) = segment.tag {
                        CodeSpan {
                            text: segment.text.to_string(),
                            tag,
                        }
                    } else {
                        span {
                            "{segment.text}"
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
    fn plaintext_is_escaped() {
        let tree = CodeTree::plaintext("<script>alert(1)</script>", "plain");
        assert_eq!(
            code_segments(tree.source(), tree.spans()),
            vec![CodeSegment {
                text: "<script>alert(1)</script>",
                tag: None,
            }]
        );
    }

    #[cfg(feature = "runtime")]
    #[test]
    fn runtime_name_detection_highlights() {
        let tree = SourceCode::new("fn main() {}")
            .with_name("main.rs")
            .into_tree();
        assert_eq!(tree.language(), Some("rust"));
        assert!(tree.spans().iter().any(|span| {
            span.tag == "k" && &tree.source()[span.start as usize..span.end as usize] == "fn"
        }));
    }

    #[cfg(feature = "runtime")]
    #[test]
    fn runtime_raw_string_uses_arborium_detection_fallback() {
        let tree = SourceCode::new("fn main() {}").into_tree();
        assert_eq!(tree.language(), None);
        assert_eq!(tree.error(), Some("could not detect language"));
    }
}
