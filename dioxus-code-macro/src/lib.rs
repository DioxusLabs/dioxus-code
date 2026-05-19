#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use macro_string::MacroString;
use proc_macro::TokenStream;
use proc_macro_crate::{FoundCrate, crate_name};
use proc_macro2::{Ident, Span, TokenStream as TokenStream2};
use quote::{format_ident, quote, quote_spanned};
use syn::parse::{Parse, ParseStream};
use syn::spanned::Spanned;
use syn::{Expr, LitStr, Token, parse_macro_input};

/// Compile-time syntax highlighting.
///
/// Reads a source file relative to the consumer's `CARGO_MANIFEST_DIR`, parses
/// it with [`arborium`], and expands to the resulting span tree. Pass the path
/// as a string literal, `concat!(...)`, or `env!(...)`. Pass
/// [`CodeOptions::builder`] with [`CodeOptions::with_language`] to name the
/// language explicitly; otherwise it is inferred from the file extension.
///
/// To highlight inline source instead of a file, use [`code_str!`].
///
/// [`CodeOptions::builder`]: https://docs.rs/dioxus-code/latest/dioxus_code/struct.CodeOptions.html#method.builder
/// [`CodeOptions::with_language`]: https://docs.rs/dioxus-code/latest/dioxus_code/struct.CodeOptions.html#method.with_language
#[proc_macro]
pub fn code(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as CodeInput);

    match expand_code(input) {
        Ok(tokens) => tokens.into(),
        Err(error) => error.to_compile_error().into(),
    }
}

/// Compile-time syntax highlighting of an inline source string.
///
/// Parses a string literal containing source code with [`arborium`] and
/// expands to the resulting span tree. Pass the source as a string literal,
/// `concat!(...)`, `include_str!(...)`, or `env!(...)`. Pass
/// [`CodeOptions::builder`] with [`CodeOptions::with_language`] to name the
/// language explicitly; otherwise, with the macro crate's `detection` feature
/// enabled, the language is inferred from the source contents.
///
/// To highlight a file on disk instead, use [`code!`].
///
/// [`CodeOptions::builder`]: https://docs.rs/dioxus-code/latest/dioxus_code/struct.CodeOptions.html#method.builder
/// [`CodeOptions::with_language`]: https://docs.rs/dioxus-code/latest/dioxus_code/struct.CodeOptions.html#method.with_language
#[proc_macro]
pub fn code_str(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as CodeStrInput);

    match expand_code_str(input) {
        Ok(tokens) => tokens.into(),
        Err(error) => error.to_compile_error().into(),
    }
}

struct CodeInput {
    path: String,
    options: Option<Expr>,
}

impl Parse for CodeInput {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let (path, options) = parse_string_and_options(input, "code macro")?;
        Ok(Self { path, options })
    }
}

struct CodeStrInput {
    source: String,
    options: Option<Expr>,
}

impl Parse for CodeStrInput {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let (source, options) = parse_string_and_options(input, "code_str macro")?;
        Ok(Self { source, options })
    }
}

fn parse_string_and_options(
    input: ParseStream<'_>,
    macro_label: &str,
) -> syn::Result<(String, Option<Expr>)> {
    let MacroString(value) = input.parse()?;
    let mut options = None;

    if input.peek(Token![,]) {
        input.parse::<Token![,]>()?;
        if !input.is_empty() {
            let expr: Expr = input.parse()?;
            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
            if !input.is_empty() {
                return Err(input.error(format!("unexpected tokens after {macro_label} options")));
            }
            options = Some(expr);
        }
    }

    Ok((value, options))
}

fn try_extract_language(expr: &Expr) -> Option<LanguageSpec> {
    match expr {
        Expr::Group(group) => try_extract_language(&group.expr),
        Expr::Paren(paren) => try_extract_language(&paren.expr),
        Expr::MethodCall(method) => {
            if method.method == "with_language"
                && method.args.len() == 1
                && let Some(slug) = try_parse_language_arg(method.args.first().unwrap())
            {
                return Some(slug);
            }
            try_extract_language(&method.receiver)
        }
        _ => None,
    }
}

fn try_parse_language_arg(expr: &Expr) -> Option<LanguageSpec> {
    match expr {
        Expr::Group(group) => try_parse_language_arg(&group.expr),
        Expr::Paren(paren) => try_parse_language_arg(&paren.expr),
        Expr::Call(call) if is_some_call(call) && call.args.len() == 1 => {
            try_parse_language_arg(call.args.first().unwrap())
        }
        Expr::Path(path) if is_none_path(path) => None,
        Expr::Path(path) => language_spec_from_path(path),
        _ => None,
    }
}

fn is_some_call(call: &syn::ExprCall) -> bool {
    let Expr::Path(path) = call.func.as_ref() else {
        return false;
    };
    path.path
        .segments
        .last()
        .is_some_and(|segment| segment.ident == "Some")
}

fn is_none_path(path: &syn::ExprPath) -> bool {
    path.path
        .segments
        .last()
        .is_some_and(|segment| segment.ident == "None")
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct LanguageSpec {
    variant: &'static str,
    slug: &'static str,
}

const LANGUAGE_VARIANTS: &[(&str, &str)] = &[
    ("Rust", "rust"),
    ("Ada", "ada"),
    ("Agda", "agda"),
    ("Asciidoc", "asciidoc"),
    ("Asm", "asm"),
    ("Awk", "awk"),
    ("Bash", "bash"),
    ("Batch", "batch"),
    ("C", "c"),
    ("CSharp", "c-sharp"),
    ("Caddy", "caddy"),
    ("Capnp", "capnp"),
    ("Cedar", "cedar"),
    ("CedarSchema", "cedarschema"),
    ("Clojure", "clojure"),
    ("CMake", "cmake"),
    ("Cobol", "cobol"),
    ("CommonLisp", "commonlisp"),
    ("Cpp", "cpp"),
    ("Css", "css"),
    ("D", "d"),
    ("Dart", "dart"),
    ("DeviceTree", "devicetree"),
    ("Diff", "diff"),
    ("Dockerfile", "dockerfile"),
    ("Dot", "dot"),
    ("Elisp", "elisp"),
    ("Elixir", "elixir"),
    ("Elm", "elm"),
    ("Erlang", "erlang"),
    ("Fish", "fish"),
    ("FSharp", "fsharp"),
    ("Gleam", "gleam"),
    ("Glsl", "glsl"),
    ("Go", "go"),
    ("GraphQL", "graphql"),
    ("Groovy", "groovy"),
    ("Haskell", "haskell"),
    ("Hcl", "hcl"),
    ("Hlsl", "hlsl"),
    ("Html", "html"),
    ("Idris", "idris"),
    ("Ini", "ini"),
    ("Java", "java"),
    ("JavaScript", "javascript"),
    ("Jinja2", "jinja2"),
    ("Jq", "jq"),
    ("Json", "json"),
    ("Julia", "julia"),
    ("Kotlin", "kotlin"),
    ("Lean", "lean"),
    ("Lua", "lua"),
    ("Markdown", "markdown"),
    ("Matlab", "matlab"),
    ("Meson", "meson"),
    ("Nginx", "nginx"),
    ("Ninja", "ninja"),
    ("Nix", "nix"),
    ("ObjectiveC", "objc"),
    ("OCaml", "ocaml"),
    ("Perl", "perl"),
    ("Php", "php"),
    ("PostScript", "postscript"),
    ("PowerShell", "powershell"),
    ("Prolog", "prolog"),
    ("Python", "python"),
    ("Query", "query"),
    ("R", "r"),
    ("Rego", "rego"),
    ("Rescript", "rescript"),
    ("Ron", "ron"),
    ("Ruby", "ruby"),
    ("Scala", "scala"),
    ("Scheme", "scheme"),
    ("Scss", "scss"),
    ("Solidity", "solidity"),
    ("Sparql", "sparql"),
    ("Sql", "sql"),
    ("SshConfig", "ssh-config"),
    ("Starlark", "starlark"),
    ("Styx", "styx"),
    ("Svelte", "svelte"),
    ("Swift", "swift"),
    ("Textproto", "textproto"),
    ("Thrift", "thrift"),
    ("TlaPlus", "tlaplus"),
    ("Toml", "toml"),
    ("Tsx", "tsx"),
    ("TypeScript", "typescript"),
    ("Typst", "typst"),
    ("Uiua", "uiua"),
    ("VisualBasic", "vb"),
    ("Verilog", "verilog"),
    ("Vhdl", "vhdl"),
    ("Vim", "vim"),
    ("Vue", "vue"),
    ("Wit", "wit"),
    ("X86Asm", "x86asm"),
    ("Xml", "xml"),
    ("Yaml", "yaml"),
    ("Yuri", "yuri"),
    ("Zig", "zig"),
    ("Zsh", "zsh"),
];

fn language_spec_from_path(path: &syn::ExprPath) -> Option<LanguageSpec> {
    let variant = path.path.segments.last()?.ident.to_string();
    language_spec_for_variant(&variant)
}

fn language_spec_for_variant(variant: &str) -> Option<LanguageSpec> {
    LANGUAGE_VARIANTS
        .iter()
        .find(|(name, _)| *name == variant)
        .map(|(variant, slug)| LanguageSpec { variant, slug })
}

fn language_spec_for_slug(slug: &str) -> Option<LanguageSpec> {
    LANGUAGE_VARIANTS
        .iter()
        .find(|(_, s)| *s == slug)
        .map(|(variant, slug)| LanguageSpec { variant, slug })
}

#[cfg(feature = "detection")]
fn detect_source_language(source: &str) -> Option<LanguageSpec> {
    betlang::detect(source)
        .language()
        .and_then(language_spec_for_betlang)
}

#[cfg(not(feature = "detection"))]
fn detect_source_language(_source: &str) -> Option<LanguageSpec> {
    None
}

#[cfg(feature = "detection")]
fn language_spec_for_betlang(language: betlang::Language) -> Option<LanguageSpec> {
    let language = match language {
        betlang::Language::Asm => "Asm",
        betlang::Language::Batch => "Batch",
        betlang::Language::C => "C",
        betlang::Language::Clojure => "Clojure",
        betlang::Language::CMake => "CMake",
        betlang::Language::Cobol => "Cobol",
        betlang::Language::Cpp => "Cpp",
        betlang::Language::Cs => "CSharp",
        betlang::Language::Css => "Css",
        betlang::Language::Dart => "Dart",
        betlang::Language::Dockerfile => "Dockerfile",
        betlang::Language::Elixir => "Elixir",
        betlang::Language::Erlang => "Erlang",
        betlang::Language::Gemfile | betlang::Language::Gemspec | betlang::Language::Ruby => "Ruby",
        betlang::Language::Go => "Go",
        betlang::Language::Gradle | betlang::Language::Groovy => "Groovy",
        betlang::Language::Haskell => "Haskell",
        betlang::Language::Html => "Html",
        betlang::Language::Ini => "Ini",
        betlang::Language::Java => "Java",
        betlang::Language::JavaScript => "JavaScript",
        betlang::Language::Json => "Json",
        betlang::Language::Julia => "Julia",
        betlang::Language::Kotlin => "Kotlin",
        betlang::Language::Lisp => "CommonLisp",
        betlang::Language::Lua => "Lua",
        betlang::Language::Markdown => "Markdown",
        betlang::Language::ObjectiveC => "ObjectiveC",
        betlang::Language::Ocaml => "OCaml",
        betlang::Language::Perl => "Perl",
        betlang::Language::Php => "Php",
        betlang::Language::Powershell => "PowerShell",
        betlang::Language::Python => "Python",
        betlang::Language::R => "R",
        betlang::Language::Rust => "Rust",
        betlang::Language::Scala => "Scala",
        betlang::Language::Shell => "Bash",
        betlang::Language::Sql => "Sql",
        betlang::Language::Swift => "Swift",
        betlang::Language::Toml => "Toml",
        betlang::Language::TypeScript => "TypeScript",
        betlang::Language::Vba => "VisualBasic",
        betlang::Language::Verilog => "Verilog",
        betlang::Language::Xml => "Xml",
        betlang::Language::Yaml => "Yaml",
        _ => return None,
    };
    language_spec_for_variant(language)
}

fn expand_code(input: CodeInput) -> syn::Result<TokenStream2> {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR")
        .map_err(|error| syn::Error::new(Span::call_site(), error.to_string()))?;
    let absolute_path = resolve_manifest_path(&PathBuf::from(manifest_dir), &input.path);
    let source = fs::read_to_string(&absolute_path).map_err(|error| {
        syn::Error::new(
            Span::call_site(),
            format!("failed to read `{}`: {error}", absolute_path.display()),
        )
    })?;

    expand_shared(input.options, source, Some(absolute_path))
}

fn expand_code_str(input: CodeStrInput) -> syn::Result<TokenStream2> {
    expand_shared(input.options, input.source, None)
}

fn expand_shared(
    options: Option<Expr>,
    source: String,
    origin_path: Option<PathBuf>,
) -> syn::Result<TokenStream2> {
    let crate_path = dioxus_code_crate_path()?;
    let options_check = options_check_tokens(&crate_path, options.as_ref());

    let Some(language) = options
        .as_ref()
        .and_then(try_extract_language)
        .or_else(|| {
            origin_path.as_ref().and_then(|path| {
                arborium::detect_language(&path.to_string_lossy()).and_then(language_spec_for_slug)
            })
        })
        .or_else(|| detect_source_language(&source))
    else {
        let message = match origin_path.as_ref() {
            Some(path) => format!(
                "could not detect language for `{}`; pass `CodeOptions::builder().with_language(Language::Rust)` or enable `detection` with the matching `lang-*` feature or `all-languages`",
                path.display()
            ),
            None => String::from(
                "could not determine language for `code_str!`; pass `CodeOptions::builder().with_language(Language::Rust)` or enable `detection` with the matching `lang-*` feature or `all-languages`",
            ),
        };
        return Ok(quote! {{
            #options_check
            compile_error!(#message);
        }});
    };

    let mut highlighter = arborium::Highlighter::new();
    let spans = highlighter
        .highlight_spans(language.slug, &source)
        .map_err(|error| syn::Error::new(Span::call_site(), error.to_string()))?;

    let variant_ident = Ident::new(language.variant, Span::call_site());

    let source_expr = match origin_path {
        Some(path) => {
            let path_lit = LitStr::new(&path.to_string_lossy(), Span::call_site());
            quote! { include_str!(#path_lit) }
        }
        None => {
            let source_lit = LitStr::new(&source, Span::call_site());
            quote! { #source_lit }
        }
    };

    let span_tokens = normalize_spans(spans).into_iter().map(|span| {
        let start = span.start;
        let end = span.end;
        let tag = LitStr::new(span.tag, Span::call_site());
        quote! {
            #crate_path::advanced::HighlightSpan::new(#start..#end, #tag)
        }
    });

    Ok(quote! {{
        #options_check
        const SOURCE: &str = #source_expr;
        const SPANS: &[#crate_path::advanced::HighlightSpan] = &[#(#span_tokens),*];
        #crate_path::advanced::HighlightedSource::from_static_parts(
            SOURCE,
            #crate_path::Language::#variant_ident,
            SPANS,
        )
    }})
}

fn options_check_tokens(crate_path: &TokenStream2, options: Option<&Expr>) -> Option<TokenStream2> {
    options.map(|expr| {
        quote_spanned! { expr.span() =>
            const _: fn() = || {
                let _: #crate_path::CodeOptions = #expr;
            };
        }
    })
}

struct NormalizedSpan {
    start: u32,
    end: u32,
    tag: &'static str,
}

struct RawSpan {
    start: u32,
    end: u32,
    tag: Option<&'static str>,
    pattern_index: u32,
}

fn normalize_spans(spans: Vec<arborium::advanced::Span>) -> Vec<NormalizedSpan> {
    use std::collections::HashMap;

    let mut deduped: HashMap<(u32, u32), RawSpan> = HashMap::new();
    for span in spans {
        let span = RawSpan {
            start: span.start,
            end: span.end,
            tag: arborium_theme::tag_for_capture(&span.capture),
            pattern_index: span.pattern_index,
        };
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
            Some(NormalizedSpan {
                start: span.start,
                end: span.end,
                tag: span.tag?,
            })
        })
        .collect();

    spans.sort_by_key(|span| (span.start, span.end));

    let mut coalesced: Vec<NormalizedSpan> = Vec::with_capacity(spans.len());
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

fn dioxus_code_crate_path() -> syn::Result<TokenStream2> {
    match crate_name("dioxus-code") {
        Ok(FoundCrate::Itself) => Ok(quote!(::dioxus_code)),
        Ok(FoundCrate::Name(name)) => {
            let ident = format_ident!("{}", name);
            Ok(quote!(::#ident))
        }
        Err(error) => Err(syn::Error::new(Span::call_site(), error.to_string())),
    }
}

fn resolve_manifest_path(manifest_dir: &Path, path: &str) -> PathBuf {
    let path_buf = PathBuf::from(path);
    if path_buf.is_absolute() && (path_buf.exists() || path_buf.starts_with(manifest_dir)) {
        return path_buf;
    }

    if let Some(stripped) = path.strip_prefix('/') {
        manifest_dir.join(stripped)
    } else {
        manifest_dir.join(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn language(expr: &str) -> Option<LanguageSpec> {
        let expr = syn::parse_str::<Expr>(expr).unwrap();
        try_extract_language(&expr)
    }

    fn slug(expr: &str) -> Option<&'static str> {
        language(expr).map(|language| language.slug)
    }

    #[test]
    fn extracts_language_variant_options() {
        assert_eq!(
            slug("CodeOptions::builder().with_language(Language::Rust)"),
            Some("rust"),
        );
        assert_eq!(
            slug("CodeOptions::builder().with_language(Some(Language::Rust))"),
            Some("rust"),
        );
    }

    #[test]
    fn extracts_none_language_option() {
        assert_eq!(slug("CodeOptions::builder().with_language(None)"), None,);
    }

    #[test]
    fn unknown_method_chains_fall_back_silently() {
        assert_eq!(slug("CodeOptions::builder()"), None);
        assert_eq!(
            slug("CodeOptions::builder().with_themes(Language::Rust)"),
            None,
        );
    }

    #[cfg(feature = "detection")]
    #[test]
    fn maps_betlang_languages_directly() {
        macro_rules! assert_betlang_mapping {
            ($betlang:expr, $variant:literal, $slug:literal) => {
                assert_eq!(
                    language_spec_for_betlang($betlang),
                    Some(LanguageSpec {
                        variant: $variant,
                        slug: $slug,
                    })
                );
            };
        }

        assert_betlang_mapping!(betlang::Language::Cs, "CSharp", "c-sharp");
        assert_betlang_mapping!(betlang::Language::Lisp, "CommonLisp", "commonlisp");
        assert_betlang_mapping!(betlang::Language::Shell, "Bash", "bash");
        assert_betlang_mapping!(betlang::Language::Vba, "VisualBasic", "vb");
    }
}
