//! Advanced building blocks for custom syntax-highlighted renderers.
//!
//! Most users should use [`Code()`](crate::Code()). This module exposes the
//! lower-level source, span, segment, and theme helpers used by companion
//! components such as `dioxus-code-editor`.
//!
//! ```rust
//! use dioxus_code::advanced::{HighlightSpan, HighlightedSource};
//! static SPANS: &[HighlightSpan] = &[HighlightSpan::new(0..2, "k")];
//! let src = HighlightedSource::from_static_parts("fn main() {}", "rust", SPANS);
//! assert_eq!(src.spans().len(), 1);
//! ```

use super::*;
use std::{borrow::Cow, ops::Range};

/// A highlighted source string with metadata and token spans.
///
/// ```rust
/// use dioxus_code::advanced::HighlightedSource;
/// let src = HighlightedSource::from_static_parts("let x = 1;", "rust", &[]);
/// assert_eq!(src.source(), "let x = 1;");
/// ```
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
    ///
    /// ```rust
    /// use dioxus_code::advanced::HighlightedSource;
    /// let src = HighlightedSource::from_static_parts("let x = 1;", "rust", &[]);
    /// assert_eq!(src.language(), Some("rust"));
    /// ```
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
    ///
    /// ```rust
    /// use dioxus_code::advanced::HighlightedSource;
    /// let src = HighlightedSource::from_static_parts("hello", "text", &[]);
    /// assert_eq!(src.source(), "hello");
    /// ```
    pub fn source(&self) -> &str {
        self.source.as_ref()
    }

    /// The detected or explicitly set language slug, if any.
    ///
    /// ```rust
    /// use dioxus_code::advanced::HighlightedSource;
    /// let src = HighlightedSource::from_static_parts("", "rust", &[]);
    /// assert_eq!(src.language(), Some("rust"));
    /// ```
    pub fn language(&self) -> Option<&str> {
        self.language.as_deref()
    }

    /// The highlighting error, if parsing fell back to plaintext.
    ///
    /// ```rust
    /// use dioxus_code::advanced::HighlightedSource;
    /// let src = HighlightedSource::from_static_parts("", "rust", &[]);
    /// assert_eq!(src.error(), None);
    /// ```
    pub fn error(&self) -> Option<&str> {
        self.error.as_deref()
    }

    /// The highlight spans covering the source.
    ///
    /// ```rust
    /// use dioxus_code::advanced::HighlightedSource;
    /// let src = HighlightedSource::from_static_parts("", "rust", &[]);
    /// assert!(src.spans().is_empty());
    /// ```
    pub fn spans(&self) -> &[HighlightSpan] {
        self.spans.as_ref()
    }

    /// Split the source into renderable highlighted segments.
    ///
    /// ```rust
    /// use dioxus_code::advanced::HighlightedSource;
    /// let src = HighlightedSource::from_static_parts("hello", "text", &[]);
    /// assert_eq!(src.segments().len(), 1);
    /// ```
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
    ///
    /// ```rust
    /// use dioxus_code::advanced::HighlightedSource;
    /// let src = HighlightedSource::from_static_parts("a\nb", "text", &[]);
    /// assert_eq!(src.lines().len(), 2);
    /// ```
    pub fn lines(&self) -> Vec<Vec<HighlightSegment<'_>>> {
        highlighted_lines(self.source(), self.spans())
    }
}

/// A highlight span attached to a byte range of source text.
///
/// ```rust
/// use dioxus_code::advanced::HighlightSpan;
/// let span = HighlightSpan::new(0..2, "k");
/// assert_eq!(span.tag(), "k");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HighlightSpan {
    start: u32,
    end: u32,
    tag: &'static str,
}

impl HighlightSpan {
    /// Create a highlight span.
    ///
    /// ```rust
    /// use dioxus_code::advanced::HighlightSpan;
    /// let span = HighlightSpan::new(0..2, "k");
    /// assert_eq!(span.range(), 0..2);
    /// ```
    pub const fn new(range: Range<u32>, tag: &'static str) -> Self {
        Self {
            start: range.start,
            end: range.end,
            tag,
        }
    }

    /// Create a highlight span from explicit byte offsets.
    ///
    /// ```rust
    /// use dioxus_code::advanced::HighlightSpan;
    /// let span = HighlightSpan::from_offsets(0, 2, "k");
    /// assert_eq!(span.start(), 0);
    /// ```
    pub const fn from_offsets(start: u32, end: u32, tag: &'static str) -> Self {
        Self { start, end, tag }
    }

    /// Byte offset, inclusive, of the span's start in the source.
    ///
    /// ```rust
    /// use dioxus_code::advanced::HighlightSpan;
    /// assert_eq!(HighlightSpan::new(3..5, "k").start(), 3);
    /// ```
    pub const fn start(self) -> u32 {
        self.start
    }

    /// Byte offset, exclusive, of the span's end in the source.
    ///
    /// ```rust
    /// use dioxus_code::advanced::HighlightSpan;
    /// assert_eq!(HighlightSpan::new(3..5, "k").end(), 5);
    /// ```
    pub const fn end(self) -> u32 {
        self.end
    }

    /// Byte range covered by this span.
    ///
    /// ```rust
    /// use dioxus_code::advanced::HighlightSpan;
    /// assert_eq!(HighlightSpan::new(3..5, "k").range(), 3..5);
    /// ```
    pub const fn range(self) -> Range<u32> {
        self.start..self.end
    }

    /// Highlight tag class suffix, for example `"k"` for keywords.
    ///
    /// ```rust
    /// use dioxus_code::advanced::HighlightSpan;
    /// assert_eq!(HighlightSpan::new(0..2, "k").tag(), "k");
    /// ```
    pub const fn tag(self) -> &'static str {
        self.tag
    }

    #[cfg(feature = "runtime")]
    pub(crate) fn set_end(&mut self, end: u32) {
        self.end = end;
    }
}

/// A borrowed render segment with an optional highlight tag.
///
/// ```rust
/// use dioxus_code::advanced::HighlightSegment;
/// let segment = HighlightSegment::new("fn", Some("k"));
/// assert_eq!(segment.text(), "fn");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HighlightSegment<'a> {
    text: &'a str,
    tag: Option<&'static str>,
}

impl<'a> HighlightSegment<'a> {
    /// Create a highlighted segment.
    ///
    /// ```rust
    /// use dioxus_code::advanced::HighlightSegment;
    /// let _segment = HighlightSegment::new("fn", Some("k"));
    /// ```
    pub const fn new(text: &'a str, tag: Option<&'static str>) -> Self {
        Self { text, tag }
    }

    /// The source text for this segment.
    ///
    /// ```rust
    /// use dioxus_code::advanced::HighlightSegment;
    /// assert_eq!(HighlightSegment::new("fn", Some("k")).text(), "fn");
    /// ```
    pub const fn text(self) -> &'a str {
        self.text
    }

    /// Highlight tag class suffix, when this segment is highlighted.
    ///
    /// ```rust
    /// use dioxus_code::advanced::HighlightSegment;
    /// assert_eq!(HighlightSegment::new("fn", Some("k")).tag(), Some("k"));
    /// ```
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
///
/// ```rust
/// use dioxus_code::advanced::TokenSpanProps;
/// let _props = TokenSpanProps { text: "fn".to_string(), tag: "k" };
/// ```
#[derive(Props, Clone, PartialEq)]
pub struct TokenSpanProps {
    /// The literal text rendered inside the span.
    #[props(into)]
    pub text: String,
    /// Highlight tag class suffix used to derive the span's class name.
    pub tag: &'static str,
}

/// Render one highlighted token as `<span class="a-{tag}">{text}</span>`.
///
/// ```rust
/// use dioxus::prelude::*;
/// use dioxus_code::advanced::TokenSpan;
///
/// fn _example() -> Element {
///     rsx! { TokenSpan { text: "fn", tag: "k" } }
/// }
/// ```
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

/// Persistent syntax highlighter that re-uses tree-sitter parse trees across edits.
///
/// `IncrementalHighlighter` keeps a parser, the most recently parsed tree, and
/// the previous source between calls. Each call to [`highlight`](Self::highlight)
/// can take a [`SourceEdit`] describing the bytes that changed; tree-sitter then
/// patches the cached tree and re-parses, reusing unmodified subtrees.
///
/// Pass `None` for the edit when the change is unknown (or when this is the
/// first parse). The previous tree is then dropped and the source is parsed
/// from scratch.
///
/// Available with the `runtime` feature. Hold one per editor instance (e.g.
/// inside `use_hook`). Languages without an incremental grammar mapping fall
/// back to the batch [`SourceCode`] path.
///
/// ```rust
/// use dioxus_code::advanced::IncrementalHighlighter;
/// let mut hl = IncrementalHighlighter::new();
/// let src = hl.highlight("fn main() {}", None, Some("rust"));
/// assert_eq!(src.language(), Some("rust"));
/// ```
#[cfg(feature = "runtime")]
#[cfg_attr(docsrs, doc(cfg(feature = "runtime")))]
pub struct IncrementalHighlighter {
    parser: arborium_tree_sitter::Parser,
    cursor: arborium_tree_sitter::QueryCursor,
    grammar: Option<GrammarSlot>,
    tree: Option<arborium_tree_sitter::Tree>,
    last_source: String,
    last_spans: Vec<HighlightSpan>,
}

#[cfg(feature = "runtime")]
struct GrammarSlot {
    slug: String,
    query: arborium_tree_sitter::Query,
}

#[cfg(feature = "runtime")]
#[cfg_attr(docsrs, doc(cfg(feature = "runtime")))]
impl Default for IncrementalHighlighter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "runtime")]
#[cfg_attr(docsrs, doc(cfg(feature = "runtime")))]
impl IncrementalHighlighter {
    /// Create a highlighter with no grammar loaded yet.
    ///
    /// ```rust
    /// use dioxus_code::advanced::IncrementalHighlighter;
    /// let _hl = IncrementalHighlighter::new();
    /// ```
    pub fn new() -> Self {
        Self {
            parser: arborium_tree_sitter::Parser::new(),
            cursor: arborium_tree_sitter::QueryCursor::new(),
            grammar: None,
            tree: None,
            last_source: String::new(),
            last_spans: Vec::new(),
        }
    }

    /// Highlight `source`, reusing the previous parse tree when possible.
    ///
    /// `language` is an explicit Arborium slug such as `"rust"`. If empty or
    /// unrecognized, the language is detected from the source content itself.
    ///
    /// Pass `Some(edit)` when the caller knows precisely which bytes changed
    /// (for example, captured from a contenteditable's `beforeinput` event).
    /// Pass `None` for a one-shot parse or when the edit is unknown — the
    /// previous tree is then discarded and `source` is parsed from scratch.
    ///
    /// ```rust
    /// use dioxus_code::advanced::IncrementalHighlighter;
    /// let mut hl = IncrementalHighlighter::new();
    /// let src = hl.highlight("fn main() {}", None, Some("rust"));
    /// assert!(!src.spans().is_empty());
    /// ```
    pub fn highlight(
        &mut self,
        source: &str,
        edit: Option<SourceEdit>,
        language: Option<&str>,
    ) -> HighlightedSource {
        let slug = language
            .map(str::to_string)
            .or_else(|| arborium::detect_language(source).map(str::to_string));

        let Some(slug) = slug else {
            self.reset();
            return HighlightedSource::plaintext(source.to_owned(), "could not detect language");
        };

        let Some((language_fn, highlights_query)) = grammar_for(&slug) else {
            self.reset();
            return batch_highlight_via_arborium(source, &slug);
        };

        let language_changed = self.grammar.as_ref().is_none_or(|g| g.slug != slug);
        if language_changed {
            let language: arborium_tree_sitter::Language = language_fn.into();
            if self.parser.set_language(&language).is_err() {
                self.reset();
                return HighlightedSource::plaintext(
                    source.to_owned(),
                    format!("failed to load grammar for {slug}"),
                );
            }
            let query = match arborium_tree_sitter::Query::new(&language, highlights_query) {
                Ok(query) => query,
                Err(error) => {
                    self.reset();
                    return HighlightedSource::plaintext(source.to_owned(), error.to_string());
                }
            };
            self.grammar = Some(GrammarSlot {
                slug: slug.clone(),
                query,
            });
            self.tree = None;
            self.last_source.clear();
            self.last_spans.clear();
        } else if self.last_source == source && self.tree.is_some() && edit.is_none() {
            return HighlightedSource::from_owned_parts(
                source.to_owned(),
                Some(slug),
                self.last_spans.clone(),
                None,
            );
        }

        // Apply the supplied edit to the cached tree so tree-sitter can reuse
        // unmodified subtrees. Without an edit (or without a tree), we drop
        // any cached tree and let the parser do a full parse.
        match (edit, self.tree.as_mut()) {
            (Some(edit), Some(tree)) => {
                if let Some(input_edit) = edit.into_input_edit(&self.last_source, source) {
                    tree.edit(&input_edit);
                } else {
                    self.tree = None;
                }
            }
            (None, _) => self.tree = None,
            _ => {}
        }

        let new_tree = match self.parser.parse(source, self.tree.as_ref()) {
            Some(tree) => tree,
            None => {
                self.reset();
                return HighlightedSource::plaintext(source.to_owned(), "tree-sitter parse failed");
            }
        };

        let grammar = self.grammar.as_ref().expect("grammar set above");
        let spans = collect_spans(&grammar.query, &mut self.cursor, &new_tree, source);

        self.tree = Some(new_tree);
        self.last_source.clear();
        self.last_source.push_str(source);
        self.last_spans = spans.clone();

        HighlightedSource::from_owned_parts(source.to_owned(), Some(slug), spans, None)
    }

    fn reset(&mut self) {
        self.grammar = None;
        self.tree = None;
        self.last_source.clear();
        self.last_spans.clear();
    }
}

#[cfg(feature = "runtime")]
fn batch_highlight_via_arborium(source: &str, slug: &str) -> HighlightedSource {
    let mut highlighter = arborium::Highlighter::new();
    match highlighter.highlight_spans(slug, source) {
        Ok(spans) => HighlightedSource::from_owned_parts(
            source.to_owned(),
            Some(slug.to_owned()),
            normalize_spans(spans.into_iter().map(RawHighlightSpan::from)),
            None,
        ),
        Err(error) => HighlightedSource::plaintext(source.to_owned(), error.to_string()),
    }
}

#[cfg(feature = "runtime")]
fn collect_spans(
    query: &arborium_tree_sitter::Query,
    cursor: &mut arborium_tree_sitter::QueryCursor,
    tree: &arborium_tree_sitter::Tree,
    source: &str,
) -> Vec<HighlightSpan> {
    use arborium_tree_sitter::StreamingIterator;

    let bytes = source.as_bytes();
    let capture_names = query.capture_names();
    let mut raw: Vec<RawHighlightSpan> = Vec::new();

    let mut matches = cursor.matches(query, tree.root_node(), bytes);
    while let Some(m) = matches.next() {
        for capture in m.captures {
            let name = capture_names[capture.index as usize];
            if name.starts_with('_') || name.starts_with("injection.") {
                continue;
            }
            raw.push(RawHighlightSpan {
                start: capture.node.start_byte() as u32,
                end: capture.node.end_byte() as u32,
                tag: arborium_theme::tag_for_capture(name),
                pattern_index: m.pattern_index as u32,
            });
        }
    }

    normalize_spans(raw)
}

#[cfg(feature = "runtime")]
fn grammar_for(slug: &str) -> Option<(arborium_tree_sitter::LanguageFn, &'static str)> {
    // Rust is bundled with the `runtime` feature; everything else is opt-in via
    // its `lang-*` cargo feature (or the `all-languages` umbrella).
    match slug {
        "rust" => Some((
            arborium::lang_rust::language(),
            arborium::lang_rust::HIGHLIGHTS_QUERY,
        )),
        #[cfg(feature = "lang-ada")]
        "ada" => Some((
            arborium::lang_ada::language(),
            arborium::lang_ada::HIGHLIGHTS_QUERY,
        )),
        #[cfg(feature = "lang-agda")]
        "agda" => Some((
            arborium::lang_agda::language(),
            arborium::lang_agda::HIGHLIGHTS_QUERY,
        )),
        #[cfg(feature = "lang-asciidoc")]
        "asciidoc" => Some((
            arborium::lang_asciidoc::language(),
            arborium::lang_asciidoc::HIGHLIGHTS_QUERY,
        )),
        #[cfg(feature = "lang-asm")]
        "asm" => Some((
            arborium::lang_asm::language(),
            arborium::lang_asm::HIGHLIGHTS_QUERY,
        )),
        #[cfg(feature = "lang-awk")]
        "awk" => Some((
            arborium::lang_awk::language(),
            arborium::lang_awk::HIGHLIGHTS_QUERY,
        )),
        #[cfg(feature = "lang-bash")]
        "bash" => Some((
            arborium::lang_bash::language(),
            arborium::lang_bash::HIGHLIGHTS_QUERY,
        )),
        #[cfg(feature = "lang-batch")]
        "batch" => Some((
            arborium::lang_batch::language(),
            arborium::lang_batch::HIGHLIGHTS_QUERY,
        )),
        #[cfg(feature = "lang-c")]
        "c" => Some((
            arborium::lang_c::language(),
            arborium::lang_c::HIGHLIGHTS_QUERY,
        )),
        #[cfg(feature = "lang-c-sharp")]
        "c-sharp" => Some((
            arborium::lang_c_sharp::language(),
            arborium::lang_c_sharp::HIGHLIGHTS_QUERY,
        )),
        #[cfg(feature = "lang-caddy")]
        "caddy" => Some((
            arborium::lang_caddy::language(),
            arborium::lang_caddy::HIGHLIGHTS_QUERY,
        )),
        #[cfg(feature = "lang-capnp")]
        "capnp" => Some((
            arborium::lang_capnp::language(),
            arborium::lang_capnp::HIGHLIGHTS_QUERY,
        )),
        #[cfg(feature = "lang-cedar")]
        "cedar" => Some((
            arborium::lang_cedar::language(),
            arborium::lang_cedar::HIGHLIGHTS_QUERY,
        )),
        #[cfg(feature = "lang-cedarschema")]
        "cedarschema" => Some((
            arborium::lang_cedarschema::language(),
            arborium::lang_cedarschema::HIGHLIGHTS_QUERY,
        )),
        #[cfg(feature = "lang-clojure")]
        "clojure" => Some((
            arborium::lang_clojure::language(),
            arborium::lang_clojure::HIGHLIGHTS_QUERY,
        )),
        #[cfg(feature = "lang-cmake")]
        "cmake" => Some((
            arborium::lang_cmake::language(),
            arborium::lang_cmake::HIGHLIGHTS_QUERY,
        )),
        #[cfg(feature = "lang-cobol")]
        "cobol" => Some((
            arborium::lang_cobol::language(),
            arborium::lang_cobol::HIGHLIGHTS_QUERY,
        )),
        #[cfg(feature = "lang-commonlisp")]
        "commonlisp" => Some((
            arborium::lang_commonlisp::language(),
            arborium::lang_commonlisp::HIGHLIGHTS_QUERY,
        )),
        #[cfg(feature = "lang-cpp")]
        "cpp" => Some((
            arborium::lang_cpp::language(),
            &arborium::lang_cpp::HIGHLIGHTS_QUERY,
        )),
        #[cfg(feature = "lang-css")]
        "css" => Some((
            arborium::lang_css::language(),
            arborium::lang_css::HIGHLIGHTS_QUERY,
        )),
        #[cfg(feature = "lang-d")]
        "d" => Some((
            arborium::lang_d::language(),
            arborium::lang_d::HIGHLIGHTS_QUERY,
        )),
        #[cfg(feature = "lang-dart")]
        "dart" => Some((
            arborium::lang_dart::language(),
            arborium::lang_dart::HIGHLIGHTS_QUERY,
        )),
        #[cfg(feature = "lang-devicetree")]
        "devicetree" => Some((
            arborium::lang_devicetree::language(),
            arborium::lang_devicetree::HIGHLIGHTS_QUERY,
        )),
        #[cfg(feature = "lang-diff")]
        "diff" => Some((
            arborium::lang_diff::language(),
            arborium::lang_diff::HIGHLIGHTS_QUERY,
        )),
        #[cfg(feature = "lang-dockerfile")]
        "dockerfile" => Some((
            arborium::lang_dockerfile::language(),
            arborium::lang_dockerfile::HIGHLIGHTS_QUERY,
        )),
        #[cfg(feature = "lang-dot")]
        "dot" => Some((
            arborium::lang_dot::language(),
            arborium::lang_dot::HIGHLIGHTS_QUERY,
        )),
        #[cfg(feature = "lang-elisp")]
        "elisp" => Some((
            arborium::lang_elisp::language(),
            arborium::lang_elisp::HIGHLIGHTS_QUERY,
        )),
        #[cfg(feature = "lang-elixir")]
        "elixir" => Some((
            arborium::lang_elixir::language(),
            arborium::lang_elixir::HIGHLIGHTS_QUERY,
        )),
        #[cfg(feature = "lang-elm")]
        "elm" => Some((
            arborium::lang_elm::language(),
            arborium::lang_elm::HIGHLIGHTS_QUERY,
        )),
        #[cfg(feature = "lang-erlang")]
        "erlang" => Some((
            arborium::lang_erlang::language(),
            arborium::lang_erlang::HIGHLIGHTS_QUERY,
        )),
        #[cfg(feature = "lang-fish")]
        "fish" => Some((
            arborium::lang_fish::language(),
            arborium::lang_fish::HIGHLIGHTS_QUERY,
        )),
        #[cfg(feature = "lang-fsharp")]
        "fsharp" => Some((
            arborium::lang_fsharp::language(),
            arborium::lang_fsharp::HIGHLIGHTS_QUERY,
        )),
        #[cfg(feature = "lang-gleam")]
        "gleam" => Some((
            arborium::lang_gleam::language(),
            arborium::lang_gleam::HIGHLIGHTS_QUERY,
        )),
        #[cfg(feature = "lang-glsl")]
        "glsl" => Some((
            arborium::lang_glsl::language(),
            &arborium::lang_glsl::HIGHLIGHTS_QUERY,
        )),
        #[cfg(feature = "lang-go")]
        "go" => Some((
            arborium::lang_go::language(),
            arborium::lang_go::HIGHLIGHTS_QUERY,
        )),
        #[cfg(feature = "lang-graphql")]
        "graphql" => Some((
            arborium::lang_graphql::language(),
            arborium::lang_graphql::HIGHLIGHTS_QUERY,
        )),
        #[cfg(feature = "lang-groovy")]
        "groovy" => Some((
            arborium::lang_groovy::language(),
            arborium::lang_groovy::HIGHLIGHTS_QUERY,
        )),
        #[cfg(feature = "lang-haskell")]
        "haskell" => Some((
            arborium::lang_haskell::language(),
            arborium::lang_haskell::HIGHLIGHTS_QUERY,
        )),
        #[cfg(feature = "lang-hcl")]
        "hcl" => Some((
            arborium::lang_hcl::language(),
            arborium::lang_hcl::HIGHLIGHTS_QUERY,
        )),
        #[cfg(feature = "lang-hlsl")]
        "hlsl" => Some((
            arborium::lang_hlsl::language(),
            &arborium::lang_hlsl::HIGHLIGHTS_QUERY,
        )),
        #[cfg(feature = "lang-html")]
        "html" => Some((
            arborium::lang_html::language(),
            arborium::lang_html::HIGHLIGHTS_QUERY,
        )),
        #[cfg(feature = "lang-idris")]
        "idris" => Some((
            arborium::lang_idris::language(),
            arborium::lang_idris::HIGHLIGHTS_QUERY,
        )),
        #[cfg(feature = "lang-ini")]
        "ini" => Some((
            arborium::lang_ini::language(),
            arborium::lang_ini::HIGHLIGHTS_QUERY,
        )),
        #[cfg(feature = "lang-java")]
        "java" => Some((
            arborium::lang_java::language(),
            arborium::lang_java::HIGHLIGHTS_QUERY,
        )),
        #[cfg(feature = "lang-javascript")]
        "javascript" => Some((
            arborium::lang_javascript::language(),
            arborium::lang_javascript::HIGHLIGHTS_QUERY,
        )),
        #[cfg(feature = "lang-jinja2")]
        "jinja2" => Some((
            arborium::lang_jinja2::language(),
            arborium::lang_jinja2::HIGHLIGHTS_QUERY,
        )),
        #[cfg(feature = "lang-jq")]
        "jq" => Some((
            arborium::lang_jq::language(),
            arborium::lang_jq::HIGHLIGHTS_QUERY,
        )),
        #[cfg(feature = "lang-json")]
        "json" => Some((
            arborium::lang_json::language(),
            arborium::lang_json::HIGHLIGHTS_QUERY,
        )),
        #[cfg(feature = "lang-julia")]
        "julia" => Some((
            arborium::lang_julia::language(),
            arborium::lang_julia::HIGHLIGHTS_QUERY,
        )),
        #[cfg(feature = "lang-kotlin")]
        "kotlin" => Some((
            arborium::lang_kotlin::language(),
            arborium::lang_kotlin::HIGHLIGHTS_QUERY,
        )),
        #[cfg(feature = "lang-lean")]
        "lean" => Some((
            arborium::lang_lean::language(),
            arborium::lang_lean::HIGHLIGHTS_QUERY,
        )),
        #[cfg(feature = "lang-lua")]
        "lua" => Some((
            arborium::lang_lua::language(),
            arborium::lang_lua::HIGHLIGHTS_QUERY,
        )),
        #[cfg(feature = "lang-markdown")]
        "markdown" => Some((
            arborium::lang_markdown::language(),
            arborium::lang_markdown::HIGHLIGHTS_QUERY,
        )),
        #[cfg(feature = "lang-matlab")]
        "matlab" => Some((
            arborium::lang_matlab::language(),
            arborium::lang_matlab::HIGHLIGHTS_QUERY,
        )),
        #[cfg(feature = "lang-meson")]
        "meson" => Some((
            arborium::lang_meson::language(),
            arborium::lang_meson::HIGHLIGHTS_QUERY,
        )),
        #[cfg(feature = "lang-nginx")]
        "nginx" => Some((
            arborium::lang_nginx::language(),
            arborium::lang_nginx::HIGHLIGHTS_QUERY,
        )),
        #[cfg(feature = "lang-ninja")]
        "ninja" => Some((
            arborium::lang_ninja::language(),
            arborium::lang_ninja::HIGHLIGHTS_QUERY,
        )),
        #[cfg(feature = "lang-nix")]
        "nix" => Some((
            arborium::lang_nix::language(),
            arborium::lang_nix::HIGHLIGHTS_QUERY,
        )),
        #[cfg(feature = "lang-objc")]
        "objc" => Some((
            arborium::lang_objc::language(),
            &arborium::lang_objc::HIGHLIGHTS_QUERY,
        )),
        #[cfg(feature = "lang-ocaml")]
        "ocaml" => Some((
            arborium::lang_ocaml::language(),
            arborium::lang_ocaml::HIGHLIGHTS_QUERY,
        )),
        #[cfg(feature = "lang-perl")]
        "perl" => Some((
            arborium::lang_perl::language(),
            arborium::lang_perl::HIGHLIGHTS_QUERY,
        )),
        #[cfg(feature = "lang-php")]
        "php" => Some((
            arborium::lang_php::language(),
            arborium::lang_php::HIGHLIGHTS_QUERY,
        )),
        #[cfg(feature = "lang-postscript")]
        "postscript" => Some((
            arborium::lang_postscript::language(),
            arborium::lang_postscript::HIGHLIGHTS_QUERY,
        )),
        #[cfg(feature = "lang-powershell")]
        "powershell" => Some((
            arborium::lang_powershell::language(),
            arborium::lang_powershell::HIGHLIGHTS_QUERY,
        )),
        #[cfg(feature = "lang-prolog")]
        "prolog" => Some((
            arborium::lang_prolog::language(),
            arborium::lang_prolog::HIGHLIGHTS_QUERY,
        )),
        #[cfg(feature = "lang-python")]
        "python" => Some((
            arborium::lang_python::language(),
            arborium::lang_python::HIGHLIGHTS_QUERY,
        )),
        #[cfg(feature = "lang-query")]
        "query" => Some((
            arborium::lang_query::language(),
            arborium::lang_query::HIGHLIGHTS_QUERY,
        )),
        #[cfg(feature = "lang-r")]
        "r" => Some((
            arborium::lang_r::language(),
            arborium::lang_r::HIGHLIGHTS_QUERY,
        )),
        #[cfg(feature = "lang-rego")]
        "rego" => Some((
            arborium::lang_rego::language(),
            arborium::lang_rego::HIGHLIGHTS_QUERY,
        )),
        #[cfg(feature = "lang-rescript")]
        "rescript" => Some((
            arborium::lang_rescript::language(),
            arborium::lang_rescript::HIGHLIGHTS_QUERY,
        )),
        #[cfg(feature = "lang-ron")]
        "ron" => Some((
            arborium::lang_ron::language(),
            arborium::lang_ron::HIGHLIGHTS_QUERY,
        )),
        #[cfg(feature = "lang-ruby")]
        "ruby" => Some((
            arborium::lang_ruby::language(),
            arborium::lang_ruby::HIGHLIGHTS_QUERY,
        )),
        #[cfg(feature = "lang-scala")]
        "scala" => Some((
            arborium::lang_scala::language(),
            arborium::lang_scala::HIGHLIGHTS_QUERY,
        )),
        #[cfg(feature = "lang-scheme")]
        "scheme" => Some((
            arborium::lang_scheme::language(),
            arborium::lang_scheme::HIGHLIGHTS_QUERY,
        )),
        #[cfg(feature = "lang-scss")]
        "scss" => Some((
            arborium::lang_scss::language(),
            &arborium::lang_scss::HIGHLIGHTS_QUERY,
        )),
        #[cfg(feature = "lang-solidity")]
        "solidity" => Some((
            arborium::lang_solidity::language(),
            arborium::lang_solidity::HIGHLIGHTS_QUERY,
        )),
        #[cfg(feature = "lang-sparql")]
        "sparql" => Some((
            arborium::lang_sparql::language(),
            arborium::lang_sparql::HIGHLIGHTS_QUERY,
        )),
        #[cfg(feature = "lang-sql")]
        "sql" => Some((
            arborium::lang_sql::language(),
            arborium::lang_sql::HIGHLIGHTS_QUERY,
        )),
        #[cfg(feature = "lang-ssh-config")]
        "ssh-config" => Some((
            arborium::lang_ssh_config::language(),
            arborium::lang_ssh_config::HIGHLIGHTS_QUERY,
        )),
        #[cfg(feature = "lang-starlark")]
        "starlark" => Some((
            arborium::lang_starlark::language(),
            arborium::lang_starlark::HIGHLIGHTS_QUERY,
        )),
        #[cfg(feature = "lang-styx")]
        "styx" => Some((
            arborium::lang_styx::language(),
            arborium::lang_styx::HIGHLIGHTS_QUERY,
        )),
        #[cfg(feature = "lang-svelte")]
        "svelte" => Some((
            arborium::lang_svelte::language(),
            &arborium::lang_svelte::HIGHLIGHTS_QUERY,
        )),
        #[cfg(feature = "lang-swift")]
        "swift" => Some((
            arborium::lang_swift::language(),
            arborium::lang_swift::HIGHLIGHTS_QUERY,
        )),
        #[cfg(feature = "lang-textproto")]
        "textproto" => Some((
            arborium::lang_textproto::language(),
            arborium::lang_textproto::HIGHLIGHTS_QUERY,
        )),
        #[cfg(feature = "lang-thrift")]
        "thrift" => Some((
            arborium::lang_thrift::language(),
            arborium::lang_thrift::HIGHLIGHTS_QUERY,
        )),
        #[cfg(feature = "lang-tlaplus")]
        "tlaplus" => Some((
            arborium::lang_tlaplus::language(),
            arborium::lang_tlaplus::HIGHLIGHTS_QUERY,
        )),
        #[cfg(feature = "lang-toml")]
        "toml" => Some((
            arborium::lang_toml::language(),
            arborium::lang_toml::HIGHLIGHTS_QUERY,
        )),
        #[cfg(feature = "lang-tsx")]
        "tsx" => Some((
            arborium::lang_tsx::language(),
            &arborium::lang_tsx::HIGHLIGHTS_QUERY,
        )),
        #[cfg(feature = "lang-typescript")]
        "typescript" => Some((
            arborium::lang_typescript::language(),
            &arborium::lang_typescript::HIGHLIGHTS_QUERY,
        )),
        #[cfg(feature = "lang-typst")]
        "typst" => Some((
            arborium::lang_typst::language(),
            arborium::lang_typst::HIGHLIGHTS_QUERY,
        )),
        #[cfg(feature = "lang-uiua")]
        "uiua" => Some((
            arborium::lang_uiua::language(),
            arborium::lang_uiua::HIGHLIGHTS_QUERY,
        )),
        #[cfg(feature = "lang-vb")]
        "vb" => Some((
            arborium::lang_vb::language(),
            arborium::lang_vb::HIGHLIGHTS_QUERY,
        )),
        #[cfg(feature = "lang-verilog")]
        "verilog" => Some((
            arborium::lang_verilog::language(),
            arborium::lang_verilog::HIGHLIGHTS_QUERY,
        )),
        #[cfg(feature = "lang-vhdl")]
        "vhdl" => Some((
            arborium::lang_vhdl::language(),
            arborium::lang_vhdl::HIGHLIGHTS_QUERY,
        )),
        #[cfg(feature = "lang-vim")]
        "vim" => Some((
            arborium::lang_vim::language(),
            arborium::lang_vim::HIGHLIGHTS_QUERY,
        )),
        #[cfg(feature = "lang-vue")]
        "vue" => Some((
            arborium::lang_vue::language(),
            &arborium::lang_vue::HIGHLIGHTS_QUERY,
        )),
        #[cfg(feature = "lang-wit")]
        "wit" => Some((
            arborium::lang_wit::language(),
            arborium::lang_wit::HIGHLIGHTS_QUERY,
        )),
        #[cfg(feature = "lang-x86asm")]
        "x86asm" => Some((
            arborium::lang_x86asm::language(),
            arborium::lang_x86asm::HIGHLIGHTS_QUERY,
        )),
        #[cfg(feature = "lang-xml")]
        "xml" => Some((
            arborium::lang_xml::language(),
            arborium::lang_xml::HIGHLIGHTS_QUERY,
        )),
        #[cfg(feature = "lang-yaml")]
        "yaml" => Some((
            arborium::lang_yaml::language(),
            arborium::lang_yaml::HIGHLIGHTS_QUERY,
        )),
        #[cfg(feature = "lang-yuri")]
        "yuri" => Some((
            arborium::lang_yuri::language(),
            arborium::lang_yuri::HIGHLIGHTS_QUERY,
        )),
        #[cfg(feature = "lang-zig")]
        "zig" => Some((
            arborium::lang_zig::language(),
            arborium::lang_zig::HIGHLIGHTS_QUERY,
        )),
        #[cfg(feature = "lang-zsh")]
        "zsh" => Some((
            arborium::lang_zsh::language(),
            arborium::lang_zsh::HIGHLIGHTS_QUERY,
        )),
        _ => None,
    }
}

/// A byte-range edit description used to drive incremental highlighting.
///
/// Build one from a real edit signal (for example a contenteditable
/// `beforeinput` event) and pass it to [`IncrementalHighlighter::highlight`].
/// The three byte offsets are relative to the source strings supplied across
/// successive calls: `start_byte` and `old_end_byte` index into the source
/// passed to the *previous* `highlight` call, while `new_end_byte` indexes
/// into the source passed to the *current* call.
///
/// ```rust
/// use dioxus_code::advanced::SourceEdit;
/// // Insertion of one byte at offset 0.
/// let _edit = SourceEdit { start_byte: 0, old_end_byte: 0, new_end_byte: 1 };
/// ```
#[cfg(feature = "runtime")]
#[cfg_attr(docsrs, doc(cfg(feature = "runtime")))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourceEdit {
    /// First byte that changed.
    pub start_byte: usize,
    /// One past the last byte of the replaced region in the previous source.
    pub old_end_byte: usize,
    /// One past the last byte of the inserted region in the new source.
    pub new_end_byte: usize,
}

#[cfg(feature = "runtime")]
impl SourceEdit {
    fn into_input_edit(
        self,
        old_source: &str,
        new_source: &str,
    ) -> Option<arborium_tree_sitter::InputEdit> {
        if self.start_byte > self.old_end_byte
            || self.start_byte > self.new_end_byte
            || self.old_end_byte > old_source.len()
            || self.new_end_byte > new_source.len()
            || !old_source.is_char_boundary(self.start_byte)
            || !old_source.is_char_boundary(self.old_end_byte)
            || !new_source.is_char_boundary(self.new_end_byte)
        {
            return None;
        }
        Some(arborium_tree_sitter::InputEdit {
            start_byte: self.start_byte,
            old_end_byte: self.old_end_byte,
            new_end_byte: self.new_end_byte,
            start_position: byte_to_point(old_source, self.start_byte),
            old_end_position: byte_to_point(old_source, self.old_end_byte),
            new_end_position: byte_to_point(new_source, self.new_end_byte),
        })
    }
}

#[cfg(feature = "runtime")]
fn byte_to_point(text: &str, byte: usize) -> arborium_tree_sitter::Point {
    let prefix = &text.as_bytes()[..byte];
    let last_newline = prefix.iter().rposition(|&b| b == b'\n');
    let row = prefix.iter().filter(|&&b| b == b'\n').count();
    let column = match last_newline {
        Some(pos) => byte - pos - 1,
        None => byte,
    };
    arborium_tree_sitter::Point { row, column }
}

/// Inject the shared syntax theme stylesheet and selected theme stylesheet.
///
/// ```rust
/// use dioxus::prelude::*;
/// use dioxus_code::{CodeTheme, Theme};
/// use dioxus_code::advanced::CodeThemeStyles;
///
/// fn _example() -> Element {
///     rsx! { CodeThemeStyles { theme: CodeTheme::fixed(Theme::TOKYO_NIGHT) } }
/// }
/// ```
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

#[cfg(all(test, feature = "runtime"))]
mod incremental_tests {
    use super::*;

    fn keyword_ranges(source: &str, hl: &HighlightedSource) -> Vec<(u32, u32, &'static str)> {
        let _ = source;
        hl.spans()
            .iter()
            .map(|s| (s.start(), s.end(), s.tag()))
            .collect()
    }

    #[test]
    fn first_parse_matches_batch_path() {
        let mut hl = IncrementalHighlighter::new();
        let source = "fn main() { let x = 1; }";
        let inc = hl.highlight(source, None, Some("rust"));
        let batch: HighlightedSource = SourceCode::new(source.to_owned())
            .with_language(Language::Rust)
            .into();

        assert_eq!(inc.language(), Some("rust"));
        assert_eq!(inc.error(), None);
        assert_eq!(keyword_ranges(source, &inc), keyword_ranges(source, &batch));
    }

    #[test]
    fn incremental_edit_with_explicit_source_edit() {
        let mut hl = IncrementalHighlighter::new();
        let first = "fn main() { let x = 1; }";
        let _ = hl.highlight(first, None, Some("rust"));

        // Insertion of a single byte "2" between the existing "1" and ";".
        let second = "fn main() { let x = 12; }";
        let edit = SourceEdit {
            start_byte: 21,
            old_end_byte: 21,
            new_end_byte: 22,
        };
        let inc = hl.highlight(second, Some(edit), Some("rust"));
        let batch: HighlightedSource = SourceCode::new(second.to_owned())
            .with_language(Language::Rust)
            .into();

        assert_eq!(keyword_ranges(second, &inc), keyword_ranges(second, &batch));
    }

    #[test]
    fn malformed_edit_falls_back_to_full_parse() {
        let mut hl = IncrementalHighlighter::new();
        let first = "fn main() { let x = 1; }";
        let _ = hl.highlight(first, None, Some("rust"));

        // old_end_byte beyond the previous source — must not panic and must
        // produce correct spans (full-parse fallback).
        let second = "fn main() { let x = 12; }";
        let edit = SourceEdit {
            start_byte: 21,
            old_end_byte: 999,
            new_end_byte: 22,
        };
        let inc = hl.highlight(second, Some(edit), Some("rust"));
        let batch: HighlightedSource = SourceCode::new(second.to_owned())
            .with_language(Language::Rust)
            .into();
        assert_eq!(keyword_ranges(second, &inc), keyword_ranges(second, &batch));
    }

    #[test]
    fn missing_edit_falls_back_to_full_parse() {
        let mut hl = IncrementalHighlighter::new();
        let _ = hl.highlight("fn main() { 1 }", None, Some("rust"));

        let updated = "fn main() { 2 }";
        let inc = hl.highlight(updated, None, Some("rust"));
        let batch: HighlightedSource = SourceCode::new(updated.to_owned())
            .with_language(Language::Rust)
            .into();
        assert_eq!(
            keyword_ranges(updated, &inc),
            keyword_ranges(updated, &batch)
        );
    }

    #[test]
    fn unchanged_source_returns_cached_spans() {
        let mut hl = IncrementalHighlighter::new();
        let source = "fn main() {}";
        let first = hl.highlight(source, None, Some("rust"));
        let second = hl.highlight(source, None, Some("rust"));
        assert_eq!(first.spans(), second.spans());
    }

    #[test]
    fn language_switch_resets_state() {
        let mut hl = IncrementalHighlighter::new();
        let _ = hl.highlight("fn main() {}", None, Some("rust"));
        let result = hl.highlight("hello", None, Some("definitely-not-a-real-language"));
        assert_eq!(result.source(), "hello");

        let after = hl.highlight("fn x() {}", None, Some("rust"));
        assert_eq!(after.language(), Some("rust"));
        assert!(
            after
                .spans()
                .iter()
                .any(|s| { &after.source()[s.start() as usize..s.end() as usize] == "fn" })
        );
    }

    #[test]
    fn byte_to_point_counts_rows_and_columns() {
        use arborium_tree_sitter::Point;
        assert_eq!(byte_to_point("abc", 2), Point { row: 0, column: 2 });
        assert_eq!(byte_to_point("ab\ncd\nef", 6), Point { row: 2, column: 0 });
        assert_eq!(byte_to_point("ab\ncd\nef", 8), Point { row: 2, column: 2 });
    }
}
