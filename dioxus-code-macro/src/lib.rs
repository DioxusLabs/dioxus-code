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

struct CodeInput {
    path: String,
    options: Option<Expr>,
}

impl Parse for CodeInput {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let MacroString(path) = input.parse()?;
        let mut options = None;

        if input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
            if !input.is_empty() {
                let expr: Expr = input.parse()?;
                if input.peek(Token![,]) {
                    input.parse::<Token![,]>()?;
                }
                if !input.is_empty() {
                    return Err(input.error("unexpected tokens after code macro options"));
                }
                options = Some(expr);
            }
        }

        Ok(Self { path, options })
    }
}

fn try_extract_language(expr: &Expr) -> Option<String> {
    match expr {
        Expr::Group(group) => try_extract_language(&group.expr),
        Expr::Paren(paren) => try_extract_language(&paren.expr),
        Expr::MethodCall(method) => {
            if method.method == "with_language" && method.args.len() == 1 {
                if let Some(slug) = try_parse_language_arg(method.args.first().unwrap()) {
                    return Some(slug);
                }
            }
            try_extract_language(&method.receiver)
        }
        _ => None,
    }
}

fn try_parse_language_arg(expr: &Expr) -> Option<String> {
    match expr {
        Expr::Group(group) => try_parse_language_arg(&group.expr),
        Expr::Paren(paren) => try_parse_language_arg(&paren.expr),
        Expr::Call(call) if is_some_call(call) && call.args.len() == 1 => {
            try_parse_language_arg(call.args.first().unwrap())
        }
        Expr::Path(path) if is_none_path(path) => None,
        Expr::Path(path) => language_slug_from_path(path).map(str::to_string),
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

fn language_slug_from_path(path: &syn::ExprPath) -> Option<&'static str> {
    let variant = path.path.segments.last()?.ident.to_string();
    LANGUAGE_VARIANTS
        .iter()
        .find(|(name, _)| *name == variant)
        .map(|(_, slug)| *slug)
}

fn language_variant_for_slug(slug: &str) -> Option<&'static str> {
    LANGUAGE_VARIANTS
        .iter()
        .find(|(_, s)| *s == slug)
        .map(|(name, _)| *name)
}

fn expand_code(input: CodeInput) -> syn::Result<TokenStream2> {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR")
        .map_err(|error| syn::Error::new(Span::call_site(), error.to_string()))?;
    let manifest_dir = PathBuf::from(manifest_dir);
    let macro_path = input.path;
    let absolute_path = resolve_manifest_path(&manifest_dir, &macro_path);
    let crate_path = dioxus_code_crate_path()?;

    let options_check = input.options.as_ref().map(|expr| {
        quote_spanned! { expr.span() =>
            const _: fn() = || {
                let _: #crate_path::CodeOptions = #expr;
            };
        }
    });

    let source = fs::read_to_string(&absolute_path).map_err(|error| {
        syn::Error::new(
            Span::call_site(),
            format!("failed to read `{}`: {error}", absolute_path.display()),
        )
    })?;

    let Some(language) = input
        .options
        .as_ref()
        .and_then(try_extract_language)
        .or_else(|| arborium::detect_language(&macro_path).map(str::to_string))
    else {
        let message = format!(
            "could not detect language for `{macro_path}`; pass `CodeOptions::builder().with_language(Language::Rust)`"
        );
        return Ok(quote! {{
            #options_check
            compile_error!(#message);
        }});
    };

    let mut highlighter = arborium::Highlighter::new();
    let spans = highlighter
        .highlight_spans(&language, &source)
        .map_err(|error| syn::Error::new(Span::call_site(), error.to_string()))?;

    let Some(variant) = language_variant_for_slug(&language) else {
        let message = format!("language `{language}` has no `Language` variant");
        return Ok(quote! {{
            #options_check
            compile_error!(#message);
        }});
    };
    let variant_ident = Ident::new(variant, Span::call_site());
    let absolute_lit = LitStr::new(&absolute_path.to_string_lossy(), Span::call_site());
    let spans = normalize_spans(spans).into_iter().map(|span| {
        let start = span.start;
        let end = span.end;
        let tag = LitStr::new(span.tag, Span::call_site());

        quote! {
            #crate_path::advanced::HighlightSpan::new(#start..#end, #tag)
        }
    });

    Ok(quote! {{
        #options_check
        const SOURCE: &str = include_str!(#absolute_lit);
        static SPANS: &[#crate_path::advanced::HighlightSpan] = &[#(#spans),*];
        #crate_path::advanced::HighlightedSource::from_static_parts(
            SOURCE,
            #crate_path::Language::#variant_ident,
            SPANS,
        )
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

    fn language(expr: &str) -> Option<String> {
        let expr = syn::parse_str::<Expr>(expr).unwrap();
        try_extract_language(&expr)
    }

    #[test]
    fn extracts_language_variant_options() {
        assert_eq!(
            language("CodeOptions::builder().with_language(Language::Rust)").as_deref(),
            Some("rust"),
        );
        assert_eq!(
            language("CodeOptions::builder().with_language(Some(Language::Rust))").as_deref(),
            Some("rust"),
        );
    }

    #[test]
    fn extracts_none_language_option() {
        assert_eq!(
            language("CodeOptions::builder().with_language(None)").as_deref(),
            None,
        );
    }

    #[test]
    fn unknown_method_chains_fall_back_silently() {
        assert_eq!(language("CodeOptions::builder()").as_deref(), None);
        assert_eq!(
            language("CodeOptions::builder().with_themes(Language::Rust)").as_deref(),
            None,
        );
    }
}
