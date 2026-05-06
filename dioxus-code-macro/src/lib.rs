#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use macro_string::MacroString;
use proc_macro::TokenStream;
use proc_macro_crate::{FoundCrate, crate_name};
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{format_ident, quote};
use syn::parse::{Parse, ParseStream};
use syn::{Expr, LitStr, Token, parse_macro_input};

/// Compile-time syntax highlighting.
///
/// Reads a source file relative to the consumer's `CARGO_MANIFEST_DIR`, parses
/// it with [`arborium`], and expands to the resulting span tree. Pass the path
/// as a string literal, `concat!(...)`, or `env!(...)`. Pass
/// `CodeOptions::builder().with_language(Language::...)` to name the language
/// explicitly; otherwise it is inferred from the file extension.
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
        language: extract_language(&expr)?,
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

fn extract_language(expr: &Expr) -> syn::Result<Option<String>> {
    match extract_language_setting(expr)? {
        LanguageSetting::Unset => Ok(None),
        LanguageSetting::Set(language) => Ok(language),
    }
}

enum LanguageSetting {
    Unset,
    Set(Option<String>),
}

fn extract_language_setting(expr: &Expr) -> syn::Result<LanguageSetting> {
    match expr {
        Expr::Call(call) if is_code_options_constructor(call) => Ok(LanguageSetting::Unset),
        Expr::Group(group) => extract_language_setting(&group.expr),
        Expr::Paren(paren) => extract_language_setting(&paren.expr),
        Expr::MethodCall(method) => {
            let _ = extract_language_setting(&method.receiver)?;
            if method.method != "with_language" {
                return Err(syn::Error::new_spanned(
                    &method.method,
                    "unsupported code option; expected `with_language(...)`",
                ));
            }

            if method.args.len() != 1 {
                return Err(syn::Error::new_spanned(
                    method,
                    "`with_language` expects exactly one argument",
                ));
            }

            Ok(LanguageSetting::Set(parse_language_arg(
                method.args.first().expect("argument length checked"),
            )?))
        }
        _ => Err(syn::Error::new_spanned(
            expr,
            "code macro options must be a `CodeOptions::builder()` chain",
        )),
    }
}

fn is_code_options_constructor(call: &syn::ExprCall) -> bool {
    if !call.args.is_empty() {
        return false;
    }

    let Expr::Path(path) = call.func.as_ref() else {
        return false;
    };
    let mut segments = path.path.segments.iter().rev();
    let Some(method) = segments.next() else {
        return false;
    };
    let Some(options) = segments.next() else {
        return false;
    };

    (method.ident == "builder" || method.ident == "new") && options.ident == "CodeOptions"
}

fn parse_language_arg(expr: &Expr) -> syn::Result<Option<String>> {
    match expr {
        Expr::Call(call) if is_some_call(call) => {
            if call.args.len() != 1 {
                return Err(syn::Error::new_spanned(
                    call,
                    "`Some` language options must contain one language",
                ));
            }
            parse_language_arg(call.args.first().expect("argument length checked")).and_then(
                |language| {
                    language
                        .ok_or_else(|| {
                            syn::Error::new_spanned(call, "`Some(None)` is not a language option")
                        })
                        .map(Some)
                },
            )
        }
        Expr::Group(group) => parse_language_arg(&group.expr),
        Expr::Paren(paren) => parse_language_arg(&paren.expr),
        Expr::Path(path) if is_none_path(path) => Ok(None),
        Expr::Path(path) => language_slug_from_path(path)
            .map(|slug| Some(slug.to_string()))
            .ok_or_else(|| unsupported_language_arg(expr)),
        _ => Err(unsupported_language_arg(expr)),
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

fn unsupported_language_arg(expr: &Expr) -> syn::Error {
    syn::Error::new_spanned(
        expr,
        "macro language must be a `Language` variant, `Some(Language::...)`, or `None`",
    )
}

fn language_slug_from_path(path: &syn::ExprPath) -> Option<&'static str> {
    let variant = path.path.segments.last()?.ident.to_string();
    match variant.as_str() {
        "Rust" => Some("rust"),
        "Ada" => Some("ada"),
        "Agda" => Some("agda"),
        "Asciidoc" => Some("asciidoc"),
        "Asm" => Some("asm"),
        "Awk" => Some("awk"),
        "Bash" => Some("bash"),
        "Batch" => Some("batch"),
        "C" => Some("c"),
        "CSharp" => Some("c-sharp"),
        "Caddy" => Some("caddy"),
        "Capnp" => Some("capnp"),
        "Cedar" => Some("cedar"),
        "CedarSchema" => Some("cedarschema"),
        "Clojure" => Some("clojure"),
        "CMake" => Some("cmake"),
        "Cobol" => Some("cobol"),
        "CommonLisp" => Some("commonlisp"),
        "Cpp" => Some("cpp"),
        "Css" => Some("css"),
        "D" => Some("d"),
        "Dart" => Some("dart"),
        "DeviceTree" => Some("devicetree"),
        "Diff" => Some("diff"),
        "Dockerfile" => Some("dockerfile"),
        "Dot" => Some("dot"),
        "Elisp" => Some("elisp"),
        "Elixir" => Some("elixir"),
        "Elm" => Some("elm"),
        "Erlang" => Some("erlang"),
        "Fish" => Some("fish"),
        "FSharp" => Some("fsharp"),
        "Gleam" => Some("gleam"),
        "Glsl" => Some("glsl"),
        "Go" => Some("go"),
        "GraphQL" => Some("graphql"),
        "Groovy" => Some("groovy"),
        "Haskell" => Some("haskell"),
        "Hcl" => Some("hcl"),
        "Hlsl" => Some("hlsl"),
        "Html" => Some("html"),
        "Idris" => Some("idris"),
        "Ini" => Some("ini"),
        "Java" => Some("java"),
        "JavaScript" => Some("javascript"),
        "Jinja2" => Some("jinja2"),
        "Jq" => Some("jq"),
        "Json" => Some("json"),
        "Julia" => Some("julia"),
        "Kotlin" => Some("kotlin"),
        "Lean" => Some("lean"),
        "Lua" => Some("lua"),
        "Markdown" => Some("markdown"),
        "Matlab" => Some("matlab"),
        "Meson" => Some("meson"),
        "Nginx" => Some("nginx"),
        "Ninja" => Some("ninja"),
        "Nix" => Some("nix"),
        "ObjectiveC" => Some("objc"),
        "OCaml" => Some("ocaml"),
        "Perl" => Some("perl"),
        "Php" => Some("php"),
        "PostScript" => Some("postscript"),
        "PowerShell" => Some("powershell"),
        "Prolog" => Some("prolog"),
        "Python" => Some("python"),
        "Query" => Some("query"),
        "R" => Some("r"),
        "Rego" => Some("rego"),
        "Rescript" => Some("rescript"),
        "Ron" => Some("ron"),
        "Ruby" => Some("ruby"),
        "Scala" => Some("scala"),
        "Scheme" => Some("scheme"),
        "Scss" => Some("scss"),
        "Solidity" => Some("solidity"),
        "Sparql" => Some("sparql"),
        "Sql" => Some("sql"),
        "SshConfig" => Some("ssh-config"),
        "Starlark" => Some("starlark"),
        "Styx" => Some("styx"),
        "Svelte" => Some("svelte"),
        "Swift" => Some("swift"),
        "Textproto" => Some("textproto"),
        "Thrift" => Some("thrift"),
        "TlaPlus" => Some("tlaplus"),
        "Toml" => Some("toml"),
        "Tsx" => Some("tsx"),
        "TypeScript" => Some("typescript"),
        "Typst" => Some("typst"),
        "Uiua" => Some("uiua"),
        "VisualBasic" => Some("vb"),
        "Verilog" => Some("verilog"),
        "Vhdl" => Some("vhdl"),
        "Vim" => Some("vim"),
        "Vue" => Some("vue"),
        "Wit" => Some("wit"),
        "X86Asm" => Some("x86asm"),
        "Xml" => Some("xml"),
        "Yaml" => Some("yaml"),
        "Yuri" => Some("yuri"),
        "Zig" => Some("zig"),
        "Zsh" => Some("zsh"),
        _ => None,
    }
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
    let Some(language) = input
        .options
        .language
        .or_else(|| arborium::detect_language(&macro_path).map(str::to_string))
    else {
        let message = format!(
            "could not detect language for `{macro_path}`; pass `CodeOptions::builder().with_language(Language::Rust)`"
        );
        return Ok(quote! {{
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
            #crate_path::advanced::HighlightSpan::new(#start..#end, #tag)
        }
    });

    Ok(quote! {{
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
        extract_language(&expr).unwrap()
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
}
