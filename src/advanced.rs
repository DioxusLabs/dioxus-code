//! Advanced building blocks for custom syntax-highlighted renderers.
//!
//! Most users should use [`Code()`](crate::Code()). This module exposes the
//! lower-level source, span, segment, and theme helpers used by companion
//! components such as `dioxus-code-editor`.
//!
//! ```rust
//! use dioxus_code::Language;
//! use dioxus_code::advanced::{HighlightSpan, HighlightedSource};
//! static SPANS: &[HighlightSpan] = &[HighlightSpan::new(0..2, "k")];
//! let src = HighlightedSource::from_static_parts("fn main() {}", Language::Rust, SPANS);
//! assert_eq!(src.spans().len(), 1);
//! ```

use super::*;
use std::{borrow::Cow, fmt, ops::Range};

/// A typed error produced while preparing runtime syntax highlighting.
///
/// These errors are recoverable for renderers: callers can surface the error
/// and render the original source as plaintext.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum HighlightError {
    /// No language could be detected and no explicit language was supplied.
    LanguageDetectionFailed,
    /// The tree-sitter parser rejected the selected grammar.
    GrammarLoad {
        /// The language whose grammar failed to load.
        language: Language,
        /// The parser's diagnostic message.
        message: String,
    },
    /// The tree-sitter highlights query failed to compile.
    Query {
        /// The language whose query failed.
        language: Language,
        /// Zero-based query row where the error was reported.
        row: usize,
        /// Zero-based query column where the error was reported.
        column: usize,
        /// Byte offset where the error was reported.
        offset: usize,
        /// The query error kind.
        kind: HighlightQueryErrorKind,
        /// The query compiler's diagnostic message.
        message: String,
    },
    /// Tree-sitter failed to parse the source.
    Parse {
        /// The language being parsed.
        language: Language,
    },
}

impl HighlightError {
    #[cfg(feature = "runtime")]
    fn grammar_load(language: Language, error: arborium_tree_sitter::LanguageError) -> Self {
        Self::GrammarLoad {
            language,
            message: error.to_string(),
        }
    }

    #[cfg(feature = "runtime")]
    fn query(language: Language, error: arborium_tree_sitter::QueryError) -> Self {
        Self::Query {
            language,
            row: error.row,
            column: error.column,
            offset: error.offset,
            kind: error.kind.into(),
            message: error.message,
        }
    }
}

impl fmt::Display for HighlightError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::LanguageDetectionFailed => write!(f, "could not detect language"),
            Self::GrammarLoad { language, message } => {
                write!(
                    f,
                    "failed to load grammar for {}: {}",
                    language.slug(),
                    message
                )
            }
            Self::Query {
                language,
                row,
                column,
                kind,
                message,
                ..
            } => {
                write!(
                    f,
                    "query error for {} at {}:{} ({}): {}",
                    language.slug(),
                    row + 1,
                    column + 1,
                    kind,
                    message
                )
            }
            Self::Parse { language } => {
                write!(f, "tree-sitter parse failed for {}", language.slug())
            }
        }
    }
}

impl std::error::Error for HighlightError {}

/// The category of a tree-sitter highlights query compilation error.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum HighlightQueryErrorKind {
    /// Invalid query syntax.
    Syntax,
    /// Invalid node type.
    NodeType,
    /// Invalid field name.
    Field,
    /// Invalid capture name.
    Capture,
    /// Invalid predicate.
    Predicate,
    /// Impossible query pattern structure.
    Structure,
    /// Query and grammar language mismatch.
    Language,
}

impl fmt::Display for HighlightQueryErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let kind = match self {
            Self::Syntax => "syntax",
            Self::NodeType => "node type",
            Self::Field => "field",
            Self::Capture => "capture",
            Self::Predicate => "predicate",
            Self::Structure => "structure",
            Self::Language => "language",
        };
        f.write_str(kind)
    }
}

#[cfg(feature = "runtime")]
impl From<arborium_tree_sitter::QueryErrorKind> for HighlightQueryErrorKind {
    fn from(kind: arborium_tree_sitter::QueryErrorKind) -> Self {
        match kind {
            arborium_tree_sitter::QueryErrorKind::Syntax => Self::Syntax,
            arborium_tree_sitter::QueryErrorKind::NodeType => Self::NodeType,
            arborium_tree_sitter::QueryErrorKind::Field => Self::Field,
            arborium_tree_sitter::QueryErrorKind::Capture => Self::Capture,
            arborium_tree_sitter::QueryErrorKind::Predicate => Self::Predicate,
            arborium_tree_sitter::QueryErrorKind::Structure => Self::Structure,
            arborium_tree_sitter::QueryErrorKind::Language => Self::Language,
        }
    }
}

/// A highlighted source string with metadata and token spans.
///
/// ```rust
/// use dioxus_code::Language;
/// use dioxus_code::advanced::HighlightedSource;
/// let src = HighlightedSource::from_static_parts("let x = 1;", Language::Rust, &[]);
/// assert_eq!(src.source(), "let x = 1;");
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HighlightedSource {
    source: Cow<'static, str>,
    language: Option<Language>,
    spans: Cow<'static, [HighlightSpan]>,
}

impl HighlightedSource {
    #[cfg(feature = "runtime")]
    pub(crate) fn from_owned_parts(
        source: String,
        language: Option<Language>,
        spans: Vec<HighlightSpan>,
    ) -> Self {
        Self {
            source: Cow::Owned(source),
            language,
            spans: Cow::Owned(spans),
        }
    }

    /// Build highlighted source from static text and spans.
    ///
    /// This is mainly useful for compile-time highlighters and macro output.
    ///
    /// ```rust
    /// use dioxus_code::Language;
    /// use dioxus_code::advanced::HighlightedSource;
    /// let src = HighlightedSource::from_static_parts("let x = 1;", Language::Rust, &[]);
    /// assert_eq!(src.language(), Some(Language::Rust));
    /// ```
    pub const fn from_static_parts(
        source: &'static str,
        language: Language,
        spans: &'static [HighlightSpan],
    ) -> Self {
        Self {
            source: Cow::Borrowed(source),
            language: Some(language),
            spans: Cow::Borrowed(spans),
        }
    }

    #[cfg(feature = "runtime")]
    pub(crate) fn plaintext(
        source: impl Into<Cow<'static, str>>,
        language: Option<Language>,
    ) -> Self {
        Self {
            source: source.into(),
            language,
            spans: Cow::Borrowed(&[]),
        }
    }

    /// The raw source text.
    ///
    /// ```rust
    /// use dioxus_code::Language;
    /// use dioxus_code::advanced::HighlightedSource;
    /// let src = HighlightedSource::from_static_parts("hello", Language::Rust, &[]);
    /// assert_eq!(src.source(), "hello");
    /// ```
    pub fn source(&self) -> &str {
        self.source.as_ref()
    }

    /// The detected or explicitly set language, if any.
    ///
    /// ```rust
    /// use dioxus_code::Language;
    /// use dioxus_code::advanced::HighlightedSource;
    /// let src = HighlightedSource::from_static_parts("", Language::Rust, &[]);
    /// assert_eq!(src.language(), Some(Language::Rust));
    /// ```
    pub const fn language(&self) -> Option<Language> {
        self.language
    }

    /// The highlight spans covering the source.
    ///
    /// ```rust
    /// use dioxus_code::Language;
    /// use dioxus_code::advanced::HighlightedSource;
    /// let src = HighlightedSource::from_static_parts("", Language::Rust, &[]);
    /// assert!(src.spans().is_empty());
    /// ```
    pub fn spans(&self) -> &[HighlightSpan] {
        self.spans.as_ref()
    }

    /// Split the source into renderable highlighted segments.
    ///
    /// ```rust
    /// use dioxus_code::Language;
    /// use dioxus_code::advanced::HighlightedSource;
    /// let src = HighlightedSource::from_static_parts("hello", Language::Rust, &[]);
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
    /// use dioxus_code::Language;
    /// use dioxus_code::advanced::HighlightedSource;
    /// let src = HighlightedSource::from_static_parts("a\nb", Language::Rust, &[]);
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

/// A live `(text, grammar)` pair you can edit, reparse, and snapshot.
///
/// `Buffer` owns the source string, the parse tree, and the highlight spans as
/// a single coherent unit. [`edit`](Self::edit) applies an incremental edit
/// (reusing the cached parse tree); [`replace`](Self::replace) swaps the source
/// wholesale; [`set_language`](Self::set_language) switches grammars and
/// reparses. After any mutation, [`source`](Self::source),
/// [`spans`](Self::spans), and [`lines`](Self::lines) reflect the new state.
///
/// Available with the `runtime` feature. Hold one per editor instance (e.g.
/// inside `use_hook`). Highlighting operations return [`HighlightError`] when
/// grammar setup or parsing fails.
///
/// ```rust
/// use dioxus_code::Language;
/// use dioxus_code::advanced::Buffer;
/// let buffer = Buffer::new(Language::Rust, "fn main() {}").expect("rust grammar loads");
/// assert_eq!(buffer.language(), Language::Rust);
/// assert!(!buffer.spans().is_empty());
/// ```
#[cfg(feature = "runtime")]
#[cfg_attr(docsrs, doc(cfg(feature = "runtime")))]
pub struct Buffer {
    parser: arborium_tree_sitter::Parser,
    cursor: arborium_tree_sitter::QueryCursor,
    language: Language,
    incremental: Option<IncrementalGrammar>,
    tree: Option<arborium_tree_sitter::Tree>,
    source: String,
    spans: Vec<HighlightSpan>,
}

#[cfg(feature = "runtime")]
struct IncrementalGrammar {
    query: arborium_tree_sitter::Query,
}

#[cfg(feature = "runtime")]
#[cfg_attr(docsrs, doc(cfg(feature = "runtime")))]
impl Buffer {
    /// Create a buffer for `language`, parse `source`, and collect spans.
    ///
    /// ```rust
    /// use dioxus_code::Language;
    /// use dioxus_code::advanced::Buffer;
    /// let buffer = Buffer::new(Language::Rust, "fn main() {}").expect("rust grammar loads");
    /// assert_eq!(buffer.source(), "fn main() {}");
    /// ```
    pub fn new(language: Language, source: impl Into<String>) -> Result<Self, HighlightError> {
        let mut buffer = Self {
            parser: arborium_tree_sitter::Parser::new(),
            cursor: arborium_tree_sitter::QueryCursor::new(),
            language,
            incremental: None,
            tree: None,
            source: String::new(),
            spans: Vec::new(),
        };
        buffer.install_grammar(language)?;
        buffer.replace(source)?;
        Ok(buffer)
    }

    /// Replace the source wholesale and reparse from scratch.
    ///
    /// Use this when the new text is unrelated to the old (e.g. loading a
    /// different file). For keystroke-level edits, prefer [`edit`](Self::edit)
    /// — it reuses the cached parse tree.
    ///
    /// ```rust
    /// use dioxus_code::Language;
    /// use dioxus_code::advanced::Buffer;
    /// let mut buffer = Buffer::new(Language::Rust, "fn old() {}").expect("rust grammar loads");
    /// buffer.replace("fn new() {}").expect("rust parses");
    /// assert_eq!(buffer.source(), "fn new() {}");
    /// ```
    pub fn replace(&mut self, source: impl Into<String>) -> Result<(), HighlightError> {
        self.source = source.into();
        self.tree = None;
        self.reparse()
    }

    /// Apply an incremental edit and reparse, reusing the cached parse tree.
    ///
    /// `new_source` must be the full text *after* the edit. `edit` describes
    /// the byte range that changed — its `start_byte` / `old_end_byte` index
    /// into the buffer's previous source, and `new_end_byte` indexes into
    /// `new_source`. If the edit is malformed, the cached tree is dropped and
    /// the new source is parsed from scratch.
    ///
    /// ```rust
    /// use dioxus_code::Language;
    /// use dioxus_code::advanced::{Buffer, SourceEdit};
    /// let mut buffer = Buffer::new(Language::Rust, "fn main() { 1 }").expect("rust grammar loads");
    /// buffer.edit(
    ///     SourceEdit { start_byte: 12, old_end_byte: 13, new_end_byte: 14 },
    ///     "fn main() { 22 }",
    /// ).expect("rust parses");
    /// assert_eq!(buffer.source(), "fn main() { 22 }");
    /// ```
    pub fn edit(
        &mut self,
        edit: SourceEdit,
        new_source: impl Into<String>,
    ) -> Result<(), HighlightError> {
        let new_source: String = new_source.into();
        if let (Some(_), Some(tree)) = (&self.incremental, self.tree.as_mut()) {
            match edit.into_input_edit(&self.source, &new_source) {
                Some(input_edit) => tree.edit(&input_edit),
                None => self.tree = None,
            }
        } else {
            self.tree = None;
        }
        self.source = new_source;
        self.reparse()
    }

    /// Switch grammars and reparse the current source.
    ///
    /// No-op when the new language matches the current one.
    ///
    /// ```rust
    /// use dioxus_code::Language;
    /// use dioxus_code::advanced::Buffer;
    /// let mut buffer = Buffer::new(Language::Rust, "fn main() {}").expect("rust grammar loads");
    /// buffer.set_language(Language::Rust).expect("rust grammar loads");
    /// assert_eq!(buffer.language(), Language::Rust);
    /// ```
    pub fn set_language(&mut self, language: Language) -> Result<(), HighlightError> {
        if self.language == language {
            return Ok(());
        }
        self.install_grammar(language)?;
        self.tree = None;
        self.reparse()
    }

    /// The current source text.
    pub fn source(&self) -> &str {
        &self.source
    }

    /// The grammar this buffer is parsing with.
    pub const fn language(&self) -> Language {
        self.language
    }

    /// Highlight spans covering the current source.
    pub fn spans(&self) -> &[HighlightSpan] {
        &self.spans
    }

    /// Split the source into renderable highlighted segments.
    pub fn segments(&self) -> Vec<HighlightSegment<'_>> {
        highlighted_segments(&self.source, &self.spans)
    }

    /// Split the source into highlighted lines, preserving trailing empty lines.
    pub fn lines(&self) -> Vec<Vec<HighlightSegment<'_>>> {
        highlighted_lines(&self.source, &self.spans)
    }

    /// Snapshot the buffer as an immutable [`HighlightedSource`].
    ///
    /// Useful for handing off to [`Code()`](crate::Code()) or any consumer
    /// that takes the frozen snapshot type.
    pub fn highlighted(&self) -> HighlightedSource {
        HighlightedSource::from_owned_parts(
            self.source.clone(),
            Some(self.language),
            self.spans.clone(),
        )
    }

    fn install_grammar(&mut self, language: Language) -> Result<(), HighlightError> {
        self.language = language;
        let (language_fn, highlights_query) = grammar_for(language);
        let ts_language: arborium_tree_sitter::Language = language_fn.into();
        if let Err(error) = self.parser.set_language(&ts_language) {
            return self.fail(HighlightError::grammar_load(language, error));
        }

        match arborium_tree_sitter::Query::new(&ts_language, highlights_query) {
            Ok(query) => {
                self.incremental = Some(IncrementalGrammar { query });
                Ok(())
            }
            Err(error) => self.fail(HighlightError::query(language, error)),
        }
    }

    fn fail<T>(&mut self, error: HighlightError) -> Result<T, HighlightError> {
        self.incremental = None;
        self.tree = None;
        self.spans.clear();
        Err(error)
    }

    fn reparse(&mut self) -> Result<(), HighlightError> {
        let Some(grammar) = &self.incremental else {
            self.tree = None;
            self.spans.clear();
            return Err(HighlightError::GrammarLoad {
                language: self.language,
                message: "grammar is not loaded".to_owned(),
            });
        };

        match self.parser.parse(&self.source, self.tree.as_ref()) {
            Some(tree) => {
                self.spans = collect_spans(&grammar.query, &mut self.cursor, &tree, &self.source);
                self.tree = Some(tree);
                Ok(())
            }
            None => self.fail(HighlightError::Parse {
                language: self.language,
            }),
        }
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
fn grammar_for(language: Language) -> (arborium_tree_sitter::LanguageFn, &'static str) {
    // Rust is bundled with the `runtime` feature; everything else is opt-in via
    // its `lang-*` cargo feature (or the `all-languages` umbrella).
    match language {
        Language::Rust => (
            arborium::lang_rust::language(),
            arborium::lang_rust::HIGHLIGHTS_QUERY,
        ),
        #[cfg(feature = "lang-ada")]
        Language::Ada => (
            arborium::lang_ada::language(),
            arborium::lang_ada::HIGHLIGHTS_QUERY,
        ),
        #[cfg(feature = "lang-agda")]
        Language::Agda => (
            arborium::lang_agda::language(),
            arborium::lang_agda::HIGHLIGHTS_QUERY,
        ),
        #[cfg(feature = "lang-asciidoc")]
        Language::Asciidoc => (
            arborium::lang_asciidoc::language(),
            arborium::lang_asciidoc::HIGHLIGHTS_QUERY,
        ),
        #[cfg(feature = "lang-asm")]
        Language::Asm => (
            arborium::lang_asm::language(),
            arborium::lang_asm::HIGHLIGHTS_QUERY,
        ),
        #[cfg(feature = "lang-awk")]
        Language::Awk => (
            arborium::lang_awk::language(),
            arborium::lang_awk::HIGHLIGHTS_QUERY,
        ),
        #[cfg(feature = "lang-bash")]
        Language::Bash => (
            arborium::lang_bash::language(),
            arborium::lang_bash::HIGHLIGHTS_QUERY,
        ),
        #[cfg(feature = "lang-batch")]
        Language::Batch => (
            arborium::lang_batch::language(),
            arborium::lang_batch::HIGHLIGHTS_QUERY,
        ),
        #[cfg(feature = "lang-c")]
        Language::C => (
            arborium::lang_c::language(),
            arborium::lang_c::HIGHLIGHTS_QUERY,
        ),
        #[cfg(feature = "lang-c-sharp")]
        Language::CSharp => (
            arborium::lang_c_sharp::language(),
            arborium::lang_c_sharp::HIGHLIGHTS_QUERY,
        ),
        #[cfg(feature = "lang-caddy")]
        Language::Caddy => (
            arborium::lang_caddy::language(),
            arborium::lang_caddy::HIGHLIGHTS_QUERY,
        ),
        #[cfg(feature = "lang-capnp")]
        Language::Capnp => (
            arborium::lang_capnp::language(),
            arborium::lang_capnp::HIGHLIGHTS_QUERY,
        ),
        #[cfg(feature = "lang-cedar")]
        Language::Cedar => (
            arborium::lang_cedar::language(),
            arborium::lang_cedar::HIGHLIGHTS_QUERY,
        ),
        #[cfg(feature = "lang-cedarschema")]
        Language::CedarSchema => (
            arborium::lang_cedarschema::language(),
            arborium::lang_cedarschema::HIGHLIGHTS_QUERY,
        ),
        #[cfg(feature = "lang-clojure")]
        Language::Clojure => (
            arborium::lang_clojure::language(),
            arborium::lang_clojure::HIGHLIGHTS_QUERY,
        ),
        #[cfg(feature = "lang-cmake")]
        Language::CMake => (
            arborium::lang_cmake::language(),
            arborium::lang_cmake::HIGHLIGHTS_QUERY,
        ),
        #[cfg(feature = "lang-cobol")]
        Language::Cobol => (
            arborium::lang_cobol::language(),
            arborium::lang_cobol::HIGHLIGHTS_QUERY,
        ),
        #[cfg(feature = "lang-commonlisp")]
        Language::CommonLisp => (
            arborium::lang_commonlisp::language(),
            arborium::lang_commonlisp::HIGHLIGHTS_QUERY,
        ),
        #[cfg(feature = "lang-cpp")]
        Language::Cpp => (
            arborium::lang_cpp::language(),
            &arborium::lang_cpp::HIGHLIGHTS_QUERY,
        ),
        #[cfg(feature = "lang-css")]
        Language::Css => (
            arborium::lang_css::language(),
            arborium::lang_css::HIGHLIGHTS_QUERY,
        ),
        #[cfg(feature = "lang-d")]
        Language::D => (
            arborium::lang_d::language(),
            arborium::lang_d::HIGHLIGHTS_QUERY,
        ),
        #[cfg(feature = "lang-dart")]
        Language::Dart => (
            arborium::lang_dart::language(),
            arborium::lang_dart::HIGHLIGHTS_QUERY,
        ),
        #[cfg(feature = "lang-devicetree")]
        Language::DeviceTree => (
            arborium::lang_devicetree::language(),
            arborium::lang_devicetree::HIGHLIGHTS_QUERY,
        ),
        #[cfg(feature = "lang-diff")]
        Language::Diff => (
            arborium::lang_diff::language(),
            arborium::lang_diff::HIGHLIGHTS_QUERY,
        ),
        #[cfg(feature = "lang-dockerfile")]
        Language::Dockerfile => (
            arborium::lang_dockerfile::language(),
            arborium::lang_dockerfile::HIGHLIGHTS_QUERY,
        ),
        #[cfg(feature = "lang-dot")]
        Language::Dot => (
            arborium::lang_dot::language(),
            arborium::lang_dot::HIGHLIGHTS_QUERY,
        ),
        #[cfg(feature = "lang-elisp")]
        Language::Elisp => (
            arborium::lang_elisp::language(),
            arborium::lang_elisp::HIGHLIGHTS_QUERY,
        ),
        #[cfg(feature = "lang-elixir")]
        Language::Elixir => (
            arborium::lang_elixir::language(),
            arborium::lang_elixir::HIGHLIGHTS_QUERY,
        ),
        #[cfg(feature = "lang-elm")]
        Language::Elm => (
            arborium::lang_elm::language(),
            arborium::lang_elm::HIGHLIGHTS_QUERY,
        ),
        #[cfg(feature = "lang-erlang")]
        Language::Erlang => (
            arborium::lang_erlang::language(),
            arborium::lang_erlang::HIGHLIGHTS_QUERY,
        ),
        #[cfg(feature = "lang-fish")]
        Language::Fish => (
            arborium::lang_fish::language(),
            arborium::lang_fish::HIGHLIGHTS_QUERY,
        ),
        #[cfg(feature = "lang-fsharp")]
        Language::FSharp => (
            arborium::lang_fsharp::language(),
            arborium::lang_fsharp::HIGHLIGHTS_QUERY,
        ),
        #[cfg(feature = "lang-gleam")]
        Language::Gleam => (
            arborium::lang_gleam::language(),
            arborium::lang_gleam::HIGHLIGHTS_QUERY,
        ),
        #[cfg(feature = "lang-glsl")]
        Language::Glsl => (
            arborium::lang_glsl::language(),
            &arborium::lang_glsl::HIGHLIGHTS_QUERY,
        ),
        #[cfg(feature = "lang-go")]
        Language::Go => (
            arborium::lang_go::language(),
            arborium::lang_go::HIGHLIGHTS_QUERY,
        ),
        #[cfg(feature = "lang-graphql")]
        Language::GraphQL => (
            arborium::lang_graphql::language(),
            arborium::lang_graphql::HIGHLIGHTS_QUERY,
        ),
        #[cfg(feature = "lang-groovy")]
        Language::Groovy => (
            arborium::lang_groovy::language(),
            arborium::lang_groovy::HIGHLIGHTS_QUERY,
        ),
        #[cfg(feature = "lang-haskell")]
        Language::Haskell => (
            arborium::lang_haskell::language(),
            arborium::lang_haskell::HIGHLIGHTS_QUERY,
        ),
        #[cfg(feature = "lang-hcl")]
        Language::Hcl => (
            arborium::lang_hcl::language(),
            arborium::lang_hcl::HIGHLIGHTS_QUERY,
        ),
        #[cfg(feature = "lang-hlsl")]
        Language::Hlsl => (
            arborium::lang_hlsl::language(),
            &arborium::lang_hlsl::HIGHLIGHTS_QUERY,
        ),
        #[cfg(feature = "lang-html")]
        Language::Html => (
            arborium::lang_html::language(),
            arborium::lang_html::HIGHLIGHTS_QUERY,
        ),
        #[cfg(feature = "lang-idris")]
        Language::Idris => (
            arborium::lang_idris::language(),
            arborium::lang_idris::HIGHLIGHTS_QUERY,
        ),
        #[cfg(feature = "lang-ini")]
        Language::Ini => (
            arborium::lang_ini::language(),
            arborium::lang_ini::HIGHLIGHTS_QUERY,
        ),
        #[cfg(feature = "lang-java")]
        Language::Java => (
            arborium::lang_java::language(),
            arborium::lang_java::HIGHLIGHTS_QUERY,
        ),
        #[cfg(feature = "lang-javascript")]
        Language::JavaScript => (
            arborium::lang_javascript::language(),
            arborium::lang_javascript::HIGHLIGHTS_QUERY,
        ),
        #[cfg(feature = "lang-jinja2")]
        Language::Jinja2 => (
            arborium::lang_jinja2::language(),
            arborium::lang_jinja2::HIGHLIGHTS_QUERY,
        ),
        #[cfg(feature = "lang-jq")]
        Language::Jq => (
            arborium::lang_jq::language(),
            arborium::lang_jq::HIGHLIGHTS_QUERY,
        ),
        #[cfg(feature = "lang-json")]
        Language::Json => (
            arborium::lang_json::language(),
            arborium::lang_json::HIGHLIGHTS_QUERY,
        ),
        #[cfg(feature = "lang-julia")]
        Language::Julia => (
            arborium::lang_julia::language(),
            arborium::lang_julia::HIGHLIGHTS_QUERY,
        ),
        #[cfg(feature = "lang-kotlin")]
        Language::Kotlin => (
            arborium::lang_kotlin::language(),
            arborium::lang_kotlin::HIGHLIGHTS_QUERY,
        ),
        #[cfg(feature = "lang-lean")]
        Language::Lean => (
            arborium::lang_lean::language(),
            arborium::lang_lean::HIGHLIGHTS_QUERY,
        ),
        #[cfg(feature = "lang-lua")]
        Language::Lua => (
            arborium::lang_lua::language(),
            arborium::lang_lua::HIGHLIGHTS_QUERY,
        ),
        #[cfg(feature = "lang-markdown")]
        Language::Markdown => (
            arborium::lang_markdown::language(),
            arborium::lang_markdown::HIGHLIGHTS_QUERY,
        ),
        #[cfg(feature = "lang-matlab")]
        Language::Matlab => (
            arborium::lang_matlab::language(),
            arborium::lang_matlab::HIGHLIGHTS_QUERY,
        ),
        #[cfg(feature = "lang-meson")]
        Language::Meson => (
            arborium::lang_meson::language(),
            arborium::lang_meson::HIGHLIGHTS_QUERY,
        ),
        #[cfg(feature = "lang-nginx")]
        Language::Nginx => (
            arborium::lang_nginx::language(),
            arborium::lang_nginx::HIGHLIGHTS_QUERY,
        ),
        #[cfg(feature = "lang-ninja")]
        Language::Ninja => (
            arborium::lang_ninja::language(),
            arborium::lang_ninja::HIGHLIGHTS_QUERY,
        ),
        #[cfg(feature = "lang-nix")]
        Language::Nix => (
            arborium::lang_nix::language(),
            arborium::lang_nix::HIGHLIGHTS_QUERY,
        ),
        #[cfg(feature = "lang-objc")]
        Language::ObjectiveC => (
            arborium::lang_objc::language(),
            &arborium::lang_objc::HIGHLIGHTS_QUERY,
        ),
        #[cfg(feature = "lang-ocaml")]
        Language::OCaml => (
            arborium::lang_ocaml::language(),
            arborium::lang_ocaml::HIGHLIGHTS_QUERY,
        ),
        #[cfg(feature = "lang-perl")]
        Language::Perl => (
            arborium::lang_perl::language(),
            arborium::lang_perl::HIGHLIGHTS_QUERY,
        ),
        #[cfg(feature = "lang-php")]
        Language::Php => (
            arborium::lang_php::language(),
            arborium::lang_php::HIGHLIGHTS_QUERY,
        ),
        #[cfg(feature = "lang-postscript")]
        Language::PostScript => (
            arborium::lang_postscript::language(),
            arborium::lang_postscript::HIGHLIGHTS_QUERY,
        ),
        #[cfg(feature = "lang-powershell")]
        Language::PowerShell => (
            arborium::lang_powershell::language(),
            arborium::lang_powershell::HIGHLIGHTS_QUERY,
        ),
        #[cfg(feature = "lang-prolog")]
        Language::Prolog => (
            arborium::lang_prolog::language(),
            arborium::lang_prolog::HIGHLIGHTS_QUERY,
        ),
        #[cfg(feature = "lang-python")]
        Language::Python => (
            arborium::lang_python::language(),
            arborium::lang_python::HIGHLIGHTS_QUERY,
        ),
        #[cfg(feature = "lang-query")]
        Language::Query => (
            arborium::lang_query::language(),
            arborium::lang_query::HIGHLIGHTS_QUERY,
        ),
        #[cfg(feature = "lang-r")]
        Language::R => (
            arborium::lang_r::language(),
            arborium::lang_r::HIGHLIGHTS_QUERY,
        ),
        #[cfg(feature = "lang-rego")]
        Language::Rego => (
            arborium::lang_rego::language(),
            arborium::lang_rego::HIGHLIGHTS_QUERY,
        ),
        #[cfg(feature = "lang-rescript")]
        Language::Rescript => (
            arborium::lang_rescript::language(),
            arborium::lang_rescript::HIGHLIGHTS_QUERY,
        ),
        #[cfg(feature = "lang-ron")]
        Language::Ron => (
            arborium::lang_ron::language(),
            arborium::lang_ron::HIGHLIGHTS_QUERY,
        ),
        #[cfg(feature = "lang-ruby")]
        Language::Ruby => (
            arborium::lang_ruby::language(),
            arborium::lang_ruby::HIGHLIGHTS_QUERY,
        ),
        #[cfg(feature = "lang-scala")]
        Language::Scala => (
            arborium::lang_scala::language(),
            arborium::lang_scala::HIGHLIGHTS_QUERY,
        ),
        #[cfg(feature = "lang-scheme")]
        Language::Scheme => (
            arborium::lang_scheme::language(),
            arborium::lang_scheme::HIGHLIGHTS_QUERY,
        ),
        #[cfg(feature = "lang-scss")]
        Language::Scss => (
            arborium::lang_scss::language(),
            &arborium::lang_scss::HIGHLIGHTS_QUERY,
        ),
        #[cfg(feature = "lang-solidity")]
        Language::Solidity => (
            arborium::lang_solidity::language(),
            arborium::lang_solidity::HIGHLIGHTS_QUERY,
        ),
        #[cfg(feature = "lang-sparql")]
        Language::Sparql => (
            arborium::lang_sparql::language(),
            arborium::lang_sparql::HIGHLIGHTS_QUERY,
        ),
        #[cfg(feature = "lang-sql")]
        Language::Sql => (
            arborium::lang_sql::language(),
            arborium::lang_sql::HIGHLIGHTS_QUERY,
        ),
        #[cfg(feature = "lang-ssh-config")]
        Language::SshConfig => (
            arborium::lang_ssh_config::language(),
            arborium::lang_ssh_config::HIGHLIGHTS_QUERY,
        ),
        #[cfg(feature = "lang-starlark")]
        Language::Starlark => (
            arborium::lang_starlark::language(),
            arborium::lang_starlark::HIGHLIGHTS_QUERY,
        ),
        #[cfg(feature = "lang-styx")]
        Language::Styx => (
            arborium::lang_styx::language(),
            arborium::lang_styx::HIGHLIGHTS_QUERY,
        ),
        #[cfg(feature = "lang-svelte")]
        Language::Svelte => (
            arborium::lang_svelte::language(),
            &arborium::lang_svelte::HIGHLIGHTS_QUERY,
        ),
        #[cfg(feature = "lang-swift")]
        Language::Swift => (
            arborium::lang_swift::language(),
            arborium::lang_swift::HIGHLIGHTS_QUERY,
        ),
        #[cfg(feature = "lang-textproto")]
        Language::Textproto => (
            arborium::lang_textproto::language(),
            arborium::lang_textproto::HIGHLIGHTS_QUERY,
        ),
        #[cfg(feature = "lang-thrift")]
        Language::Thrift => (
            arborium::lang_thrift::language(),
            arborium::lang_thrift::HIGHLIGHTS_QUERY,
        ),
        #[cfg(feature = "lang-tlaplus")]
        Language::TlaPlus => (
            arborium::lang_tlaplus::language(),
            arborium::lang_tlaplus::HIGHLIGHTS_QUERY,
        ),
        #[cfg(feature = "lang-toml")]
        Language::Toml => (
            arborium::lang_toml::language(),
            arborium::lang_toml::HIGHLIGHTS_QUERY,
        ),
        #[cfg(feature = "lang-tsx")]
        Language::Tsx => (
            arborium::lang_tsx::language(),
            &arborium::lang_tsx::HIGHLIGHTS_QUERY,
        ),
        #[cfg(feature = "lang-typescript")]
        Language::TypeScript => (
            arborium::lang_typescript::language(),
            &arborium::lang_typescript::HIGHLIGHTS_QUERY,
        ),
        #[cfg(feature = "lang-typst")]
        Language::Typst => (
            arborium::lang_typst::language(),
            arborium::lang_typst::HIGHLIGHTS_QUERY,
        ),
        #[cfg(feature = "lang-uiua")]
        Language::Uiua => (
            arborium::lang_uiua::language(),
            arborium::lang_uiua::HIGHLIGHTS_QUERY,
        ),
        #[cfg(feature = "lang-vb")]
        Language::VisualBasic => (
            arborium::lang_vb::language(),
            arborium::lang_vb::HIGHLIGHTS_QUERY,
        ),
        #[cfg(feature = "lang-verilog")]
        Language::Verilog => (
            arborium::lang_verilog::language(),
            arborium::lang_verilog::HIGHLIGHTS_QUERY,
        ),
        #[cfg(feature = "lang-vhdl")]
        Language::Vhdl => (
            arborium::lang_vhdl::language(),
            arborium::lang_vhdl::HIGHLIGHTS_QUERY,
        ),
        #[cfg(feature = "lang-vim")]
        Language::Vim => (
            arborium::lang_vim::language(),
            arborium::lang_vim::HIGHLIGHTS_QUERY,
        ),
        #[cfg(feature = "lang-vue")]
        Language::Vue => (
            arborium::lang_vue::language(),
            &arborium::lang_vue::HIGHLIGHTS_QUERY,
        ),
        #[cfg(feature = "lang-wit")]
        Language::Wit => (
            arborium::lang_wit::language(),
            arborium::lang_wit::HIGHLIGHTS_QUERY,
        ),
        #[cfg(feature = "lang-x86asm")]
        Language::X86Asm => (
            arborium::lang_x86asm::language(),
            arborium::lang_x86asm::HIGHLIGHTS_QUERY,
        ),
        #[cfg(feature = "lang-xml")]
        Language::Xml => (
            arborium::lang_xml::language(),
            arborium::lang_xml::HIGHLIGHTS_QUERY,
        ),
        #[cfg(feature = "lang-yaml")]
        Language::Yaml => (
            arborium::lang_yaml::language(),
            arborium::lang_yaml::HIGHLIGHTS_QUERY,
        ),
        #[cfg(feature = "lang-yuri")]
        Language::Yuri => (
            arborium::lang_yuri::language(),
            arborium::lang_yuri::HIGHLIGHTS_QUERY,
        ),
        #[cfg(feature = "lang-zig")]
        Language::Zig => (
            arborium::lang_zig::language(),
            arborium::lang_zig::HIGHLIGHTS_QUERY,
        ),
        #[cfg(feature = "lang-zsh")]
        Language::Zsh => (
            arborium::lang_zsh::language(),
            arborium::lang_zsh::HIGHLIGHTS_QUERY,
        ),
    }
}

/// A byte-range edit description used to drive incremental highlighting.
///
/// Build one from a real edit signal (for example a contenteditable
/// `beforeinput` event) and pass it to [`Buffer::edit`]. `start_byte` and
/// `old_end_byte` index into the buffer's previous source, while
/// `new_end_byte` indexes into the new source supplied alongside the edit.
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
mod buffer_tests {
    use super::*;

    fn span_ranges(spans: &[HighlightSpan]) -> Vec<(u32, u32, &'static str)> {
        spans
            .iter()
            .map(|s| (s.start(), s.end(), s.tag()))
            .collect()
    }

    fn batch_spans(source: &str, language: Language) -> Vec<HighlightSpan> {
        let snapshot: HighlightedSource = SourceCode::new(source.to_owned())
            .with_language(language)
            .into();
        snapshot.spans().to_vec()
    }

    #[test]
    fn new_matches_batch_path() {
        let source = "fn main() { let x = 1; }";
        let buffer = Buffer::new(Language::Rust, source).unwrap();

        assert_eq!(buffer.language(), Language::Rust);
        assert_eq!(
            span_ranges(buffer.spans()),
            span_ranges(&batch_spans(source, Language::Rust)),
        );
    }

    #[test]
    fn edit_with_explicit_source_edit() {
        let mut buffer = Buffer::new(Language::Rust, "fn main() { let x = 1; }").unwrap();
        let updated = "fn main() { let x = 12; }";
        buffer
            .edit(
                SourceEdit {
                    start_byte: 21,
                    old_end_byte: 21,
                    new_end_byte: 22,
                },
                updated,
            )
            .unwrap();

        assert_eq!(buffer.source(), updated);
        assert_eq!(
            span_ranges(buffer.spans()),
            span_ranges(&batch_spans(updated, Language::Rust)),
        );
    }

    #[test]
    fn malformed_edit_falls_back_to_full_parse() {
        let mut buffer = Buffer::new(Language::Rust, "fn main() { let x = 1; }").unwrap();
        let updated = "fn main() { let x = 12; }";
        buffer
            .edit(
                // old_end_byte beyond the previous source — must not panic.
                SourceEdit {
                    start_byte: 21,
                    old_end_byte: 999,
                    new_end_byte: 22,
                },
                updated,
            )
            .unwrap();

        assert_eq!(
            span_ranges(buffer.spans()),
            span_ranges(&batch_spans(updated, Language::Rust)),
        );
    }

    #[test]
    fn replace_drops_cached_tree() {
        let mut buffer = Buffer::new(Language::Rust, "fn main() { 1 }").unwrap();
        let updated = "fn main() { 2 }";
        buffer.replace(updated).unwrap();

        assert_eq!(buffer.source(), updated);
        assert_eq!(
            span_ranges(buffer.spans()),
            span_ranges(&batch_spans(updated, Language::Rust)),
        );
    }

    #[test]
    fn set_language_reparses() {
        let mut buffer = Buffer::new(Language::Rust, "fn main() {}").unwrap();
        let rust_spans = buffer.spans().to_vec();

        // Re-set to the same language is a no-op but should still produce
        // the same output.
        buffer.set_language(Language::Rust).unwrap();
        assert_eq!(buffer.spans(), rust_spans.as_slice());
    }

    #[test]
    fn byte_to_point_counts_rows_and_columns() {
        use arborium_tree_sitter::Point;
        assert_eq!(byte_to_point("abc", 2), Point { row: 0, column: 2 });
        assert_eq!(byte_to_point("ab\ncd\nef", 6), Point { row: 2, column: 0 });
        assert_eq!(byte_to_point("ab\ncd\nef", 8), Point { row: 2, column: 2 });
    }
}
