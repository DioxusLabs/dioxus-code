#![doc = include_str!("../../README.md")]
#![warn(missing_docs)]

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use proc_macro::TokenStream;
use proc_macro_crate::{FoundCrate, crate_name};
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{format_ident, quote};
use syn::parse::{Parse, ParseStream};
use syn::{Expr, Ident, LitStr, Token, parse_macro_input};

/// Compile-time syntax highlighting.
///
/// Reads a source file relative to the consumer's `CARGO_MANIFEST_DIR`, parses
/// it with [`arborium`], and expands to the resulting span tree. Pass the path
/// as a string literal, `concat!(...)`, or `env!(...)`. Optionally name the
/// language explicitly with `code!("/path", "rust")`; otherwise it is inferred
/// from the file extension.
#[proc_macro]
pub fn code(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as CodeInput);

    match expand_code(input) {
        Ok(tokens) => tokens.into(),
        Err(error) => error.to_compile_error().into(),
    }
}

struct CodeInput {
    path: Expr,
    language: Option<LitStr>,
}

impl Parse for CodeInput {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let path = input.parse()?;
        let mut language = None;

        if input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
            if !input.is_empty() {
                let ident = input.parse::<Ident>()?;
                if ident != "language" {
                    return Err(syn::Error::new(
                        ident.span(),
                        "expected `language = \"...\"`",
                    ));
                }
                input.parse::<Token![=]>()?;
                language = Some(input.parse()?);
            }
        }

        Ok(Self { path, language })
    }
}

fn expand_code(input: CodeInput) -> syn::Result<TokenStream2> {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR")
        .map_err(|error| syn::Error::new(Span::call_site(), error.to_string()))?;
    let manifest_dir = PathBuf::from(manifest_dir);
    let macro_path = eval_path_expr(&input.path)?;
    let absolute_path = resolve_manifest_path(&manifest_dir, &macro_path);

    let source = fs::read_to_string(&absolute_path).map_err(|error| {
        syn::Error::new(
            Span::call_site(),
            format!("failed to read `{}`: {error}", absolute_path.display()),
        )
    })?;

    let language = input
        .language
        .as_ref()
        .map(LitStr::value)
        .or_else(|| arborium::detect_language(&macro_path).map(str::to_string))
        .ok_or_else(|| {
            syn::Error::new(
                Span::call_site(),
                format!("could not detect language for `{macro_path}`; pass `language = \"...\"`"),
            )
        })?;

    let mut highlighter = arborium::Highlighter::new();
    let spans = highlighter
        .highlight_spans(&language, &source)
        .map_err(|error| syn::Error::new(Span::call_site(), error.to_string()))?;
    let crate_path = dioxus_code_crate_path()?;

    let language_lit = LitStr::new(&language, Span::call_site());
    let absolute_lit = LitStr::new(&absolute_path.to_string_lossy(), Span::call_site());
    let spans = normalize_spans(spans).into_iter().map(|span| {
        let start = span.start;
        let end = span.end;
        let tag = LitStr::new(span.tag, Span::call_site());

        quote! {
            #crate_path::StaticSpan {
                start: #start,
                end: #end,
                tag: #tag,
            }
        }
    });

    Ok(quote! {{
        const SOURCE: &str = include_str!(#absolute_lit);
        static SPANS: &[#crate_path::StaticSpan] = &[#(#spans),*];
        #crate_path::CodeTree::from_static_parts(SOURCE, #language_lit, SPANS)
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
    if let Some(stripped) = path.strip_prefix('/') {
        manifest_dir.join(stripped)
    } else {
        manifest_dir.join(path)
    }
}

fn eval_path_expr(expr: &Expr) -> syn::Result<String> {
    match expr {
        Expr::Lit(expr_lit) => {
            if let syn::Lit::Str(lit) = &expr_lit.lit {
                Ok(lit.value())
            } else {
                Err(syn::Error::new_spanned(
                    expr,
                    "path must be a string literal",
                ))
            }
        }
        Expr::Macro(expr_macro) => {
            let Some(ident) = expr_macro.mac.path.get_ident() else {
                return Err(syn::Error::new_spanned(
                    expr,
                    "only string literals, concat!, and env! are supported",
                ));
            };

            match ident.to_string().as_str() {
                "concat" => eval_concat(expr_macro.mac.tokens.clone()),
                "env" => eval_env(expr_macro.mac.tokens.clone()),
                _ => Err(syn::Error::new_spanned(
                    expr,
                    "only string literals, concat!, and env! are supported",
                )),
            }
        }
        _ => Err(syn::Error::new_spanned(
            expr,
            "only string literals, concat!, and env! are supported",
        )),
    }
}

fn eval_concat(tokens: TokenStream2) -> syn::Result<String> {
    struct Args {
        exprs: syn::punctuated::Punctuated<Expr, Token![,]>,
    }

    impl Parse for Args {
        fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
            Ok(Self {
                exprs: syn::punctuated::Punctuated::parse_terminated(input)?,
            })
        }
    }

    let args = syn::parse2::<Args>(tokens)?;
    let mut value = String::new();
    for expr in args.exprs {
        value.push_str(&eval_path_expr(&expr)?);
    }
    Ok(value)
}

fn eval_env(tokens: TokenStream2) -> syn::Result<String> {
    let lit = syn::parse2::<LitStr>(tokens)?;
    env::var(lit.value()).map_err(|error| syn::Error::new(lit.span(), error.to_string()))
}
