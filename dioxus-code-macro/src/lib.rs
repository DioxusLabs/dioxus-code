#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use macro_string::MacroString;
use proc_macro::TokenStream;
use proc_macro_crate::{FoundCrate, crate_name};
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{ToTokens, format_ident, quote};
use syn::parse::{Parse, ParseStream};
use syn::{Expr, LitStr, Token, parse_macro_input};

/// Compile-time syntax highlighting.
///
/// Reads a source file relative to the consumer's `CARGO_MANIFEST_DIR`, parses
/// it with [`arborium`], and expands to the resulting span tree. Pass the path
/// as a string literal, `concat!(...)`, or `env!(...)`. Pass
/// `CodeOptions::new().with_language("...")` to name the language explicitly;
/// otherwise it is inferred from the file extension.
#[proc_macro]
pub fn code(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as CodeInput);

    match expand_code(input) {
        Ok(tokens) => tokens.into(),
        Err(error) => error.to_compile_error().into(),
    }
}

struct CodeInput {
    path: String,
    options: CodeOptionsInput,
}

#[derive(Default)]
struct CodeOptionsInput {
    language: Option<String>,
    validation_tokens: Option<TokenStream2>,
}

impl Parse for CodeInput {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let MacroString(path) = input.parse()?;
        let mut options = CodeOptionsInput::default();

        if input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
            if !input.is_empty() {
                options = parse_code_options(input)?;
            }
        }

        Ok(Self { path, options })
    }
}

fn parse_code_options(input: ParseStream<'_>) -> syn::Result<CodeOptionsInput> {
    let expr = input.parse::<Expr>()?;
    parse_optional_trailing_comma(input)?;
    Ok(CodeOptionsInput {
        language: extract_language(&expr),
        validation_tokens: Some(expr.to_token_stream()),
    })
}

fn parse_optional_trailing_comma(input: ParseStream<'_>) -> syn::Result<()> {
    if input.peek(Token![,]) {
        input.parse::<Token![,]>()?;
    }
    if input.is_empty() {
        Ok(())
    } else {
        Err(input.error("unexpected tokens after code macro options"))
    }
}

fn extract_language(expr: &Expr) -> Option<String> {
    let Expr::MethodCall(method) = expr else {
        return None;
    };

    let receiver_language = extract_language(&method.receiver);
    if method.method != "with_language" || method.args.len() != 1 {
        return receiver_language;
    }

    method
        .args
        .first()
        .and_then(|arg| eval_string_expr(arg).ok())
        .or(receiver_language)
}

fn eval_string_expr(expr: &Expr) -> syn::Result<String> {
    syn::parse2::<MacroString>(expr.to_token_stream()).map(|MacroString(value)| value)
}

fn expand_code(input: CodeInput) -> syn::Result<TokenStream2> {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR")
        .map_err(|error| syn::Error::new(Span::call_site(), error.to_string()))?;
    let manifest_dir = PathBuf::from(manifest_dir);
    let macro_path = input.path;
    let absolute_path = resolve_manifest_path(&manifest_dir, &macro_path);

    let source = fs::read_to_string(&absolute_path).map_err(|error| {
        syn::Error::new(
            Span::call_site(),
            format!("failed to read `{}`: {error}", absolute_path.display()),
        )
    })?;

    let crate_path = dioxus_code_crate_path()?;
    let options_validation = input.options.validation_tokens.map(|options| {
        quote! {
            const _: #crate_path::CodeOptions = #options;
        }
    });
    let Some(language) = input
        .options
        .language
        .or_else(|| arborium::detect_language(&macro_path).map(str::to_string))
    else {
        let message = format!(
            "could not detect language for `{macro_path}`; pass `CodeOptions::new().with_language(\"rust\")`"
        );
        return Ok(quote! {{
            #options_validation
            compile_error!(#message);
        }});
    };

    let mut highlighter = arborium::Highlighter::new();
    let spans = highlighter
        .highlight_spans(&language, &source)
        .map_err(|error| syn::Error::new(Span::call_site(), error.to_string()))?;

    let language_lit = LitStr::new(&language, Span::call_site());
    let absolute_lit = LitStr::new(&absolute_path.to_string_lossy(), Span::call_site());
    let spans = normalize_spans(spans).into_iter().map(|span| {
        let start = span.start;
        let end = span.end;
        let tag = LitStr::new(span.tag, Span::call_site());

        quote! {
            #crate_path::advanced::HighlightSpan::new(#start, #end, #tag)
        }
    });

    Ok(quote! {{
        #options_validation
        const SOURCE: &str = include_str!(#absolute_lit);
        static SPANS: &[#crate_path::advanced::HighlightSpan] = &[#(#spans),*];
        #crate_path::advanced::HighlightedSource::from_static_parts(SOURCE, #language_lit, SPANS)
    }})
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
        Ok(FoundCrate::Itself) => Ok(quote!(crate)),
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
