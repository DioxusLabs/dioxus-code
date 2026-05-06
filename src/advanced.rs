//! Advanced building blocks for custom syntax-highlighted renderers.
//!
//! Most users should use [`Code()`](crate::Code()). This module exposes the
//! lower-level source, span, segment, and theme helpers used by companion
//! components such as `dioxus-code-editor`.

use super::*;
use std::borrow::Cow;

/// A highlighted source string with metadata and token spans.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HighlightedSource {
    source: Cow<'static, str>,
    language: Option<Cow<'static, str>>,
    spans: Cow<'static, [HighlightSpan]>,
    error: Option<Cow<'static, str>>,
}

impl HighlightedSource {
    #[cfg(feature = "runtime")]
    pub(crate) fn from_owned_parts(
        source: String,
        language: Option<String>,
        spans: Vec<HighlightSpan>,
        error: Option<String>,
    ) -> Self {
        Self {
            source: Cow::Owned(source),
            language: language.map(Cow::Owned),
            spans: Cow::Owned(spans),
            error: error.map(Cow::Owned),
        }
    }

    /// Build highlighted source from static text and spans.
    ///
    /// This is mainly useful for compile-time highlighters and macro output.
    pub const fn from_static_parts(
        source: &'static str,
        language: &'static str,
        spans: &'static [HighlightSpan],
    ) -> Self {
        Self {
            source: Cow::Borrowed(source),
            language: Some(Cow::Borrowed(language)),
            spans: Cow::Borrowed(spans),
            error: None,
        }
    }

    #[cfg(feature = "runtime")]
    pub(crate) fn plaintext(
        source: impl Into<Cow<'static, str>>,
        error: impl Into<Cow<'static, str>>,
    ) -> Self {
        Self {
            source: source.into(),
            language: None,
            spans: Cow::Borrowed(&[]),
            error: Some(error.into()),
        }
    }

    /// The raw source text.
    pub fn source(&self) -> &str {
        self.source.as_ref()
    }

    /// The detected or explicitly set language slug, if any.
    pub fn language(&self) -> Option<&str> {
        self.language.as_deref()
    }

    /// The highlighting error, if parsing fell back to plaintext.
    pub fn error(&self) -> Option<&str> {
        self.error.as_deref()
    }

    /// The highlight spans covering the source.
    pub fn spans(&self) -> &[HighlightSpan] {
        self.spans.as_ref()
    }

    /// Split the source into renderable highlighted segments.
    pub fn segments(&self) -> Vec<HighlightSegment<'_>> {
        highlighted_segments(self.source(), self.spans())
    }

    pub(crate) fn trimmed_segments(&self) -> Vec<HighlightSegment<'_>> {
        highlighted_segments(self.source().trim_end_matches('\n'), self.spans())
    }

    /// Split the source into highlighted lines.
    ///
    /// Trailing empty lines are preserved so editor renderers can keep line
    /// numbers and input rows aligned with the original source text.
    pub fn lines(&self) -> Vec<Vec<HighlightSegment<'_>>> {
        highlighted_lines(self.source(), self.spans())
    }
}

/// A highlight span attached to a byte range of source text.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HighlightSpan {
    start: u32,
    end: u32,
    tag: &'static str,
}

impl HighlightSpan {
    /// Create a highlight span.
    pub const fn new(start: u32, end: u32, tag: &'static str) -> Self {
        Self { start, end, tag }
    }

    /// Byte offset, inclusive, of the span's start in the source.
    pub const fn start(self) -> u32 {
        self.start
    }

    /// Byte offset, exclusive, of the span's end in the source.
    pub const fn end(self) -> u32 {
        self.end
    }

    /// Highlight tag class suffix, for example `"k"` for keywords.
    pub const fn tag(self) -> &'static str {
        self.tag
    }

    #[cfg(feature = "runtime")]
    pub(crate) fn set_end(&mut self, end: u32) {
        self.end = end;
    }
}

/// A borrowed render segment with an optional highlight tag.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HighlightSegment<'a> {
    text: &'a str,
    tag: Option<&'static str>,
}

impl<'a> HighlightSegment<'a> {
    /// Create a highlighted segment.
    pub const fn new(text: &'a str, tag: Option<&'static str>) -> Self {
        Self { text, tag }
    }

    /// The source text for this segment.
    pub const fn text(self) -> &'a str {
        self.text
    }

    /// Highlight tag class suffix, when this segment is highlighted.
    pub const fn tag(self) -> Option<&'static str> {
        self.tag
    }
}

fn highlighted_segments<'a>(source: &'a str, spans: &[HighlightSpan]) -> Vec<HighlightSegment<'a>> {
    if spans.is_empty() {
        return vec![HighlightSegment::new(source, None)];
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
            segments.push(HighlightSegment::new(
                &source[last_pos..pos],
                stack.last().map(|&i| spans[i].tag),
            ));
            last_pos = pos;
        }

        if is_start {
            stack.push(span_index);
        } else if let Some(index) = stack.iter().rposition(|&i| i == span_index) {
            stack.remove(index);
        }
    }

    if last_pos < source.len() {
        segments.push(HighlightSegment::new(
            &source[last_pos..],
            stack.last().map(|&i| spans[i].tag),
        ));
    }

    segments
}

fn highlighted_lines<'a>(
    source: &'a str,
    spans: &[HighlightSpan],
) -> Vec<Vec<HighlightSegment<'a>>> {
    let mut lines = vec![Vec::new()];

    for segment in highlighted_segments(source, spans) {
        push_line_segments(&mut lines, segment);
    }

    lines
}

fn push_line_segments<'a>(
    lines: &mut Vec<Vec<HighlightSegment<'a>>>,
    segment: HighlightSegment<'a>,
) {
    let mut text = segment.text;

    loop {
        if let Some(newline) = text.find('\n') {
            let before_newline = &text[..newline];
            if !before_newline.is_empty() {
                lines
                    .last_mut()
                    .unwrap()
                    .push(HighlightSegment::new(before_newline, segment.tag));
            }
            lines.push(Vec::new());
            text = &text[newline + 1..];
        } else {
            if !text.is_empty() {
                lines
                    .last_mut()
                    .unwrap()
                    .push(HighlightSegment::new(text, segment.tag));
            }
            break;
        }
    }
}

/// Props for [`TokenSpan`].
#[derive(Props, Clone, PartialEq)]
pub struct TokenSpanProps {
    /// The literal text rendered inside the span.
    pub text: String,
    /// Highlight tag class suffix used to derive the span's class name.
    pub tag: &'static str,
}

/// Render one highlighted token as `<span class="a-{tag}">{text}</span>`.
#[component]
pub fn TokenSpan(props: TokenSpanProps) -> Element {
    let class = format!("a-{}", props.tag);
    rsx! {
        span {
            class,
            "{props.text}"
        }
    }
}

/// Inject the shared syntax theme stylesheet and selected theme stylesheet.
#[component]
pub fn CodeThemeStyles(theme: CodeTheme) -> Element {
    let shared_theme_css = Theme::THEME_CSS;

    match theme.stylesheets() {
        CodeThemeStylesheets::Fixed(stylesheet) => {
            let theme_asset = stylesheet.asset;
            let theme_key = stylesheet.class;

            rsx! {
                document::Stylesheet { href: shared_theme_css }
                {rsx!{document::Stylesheet { key: "{theme_key}", href: theme_asset }}}
            }
        }
        CodeThemeStylesheets::System { light, dark } => {
            let light_asset = light.asset;
            let dark_asset = dark.asset;
            let light_key = light.class;
            let dark_key = dark.class;

            rsx! {
                document::Stylesheet { href: shared_theme_css }
                {rsx!{document::Stylesheet { key: "{light_key}", href: light_asset }}}
                {rsx!{document::Stylesheet { key: "{dark_key}", href: dark_asset }}}
            }
        }
    }
}
