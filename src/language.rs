//! Tree-sitter grammar selection.
//!
//! [`Language`] is a closed set of Arborium language slugs. Each named variant
//! is gated by the same cargo feature as the corresponding grammar in
//! `dioxus-code`, so the enum exposes the variants compiled into this build.

macro_rules! define_languages {
    (
        $(
            $(#[$attr:meta])*
            $variant:ident => $slug:literal
        ),* $(,)?
    ) => {
        /// Tree-sitter grammar identifier.
        ///
        /// Each variant maps to an Arborium language slug via
        /// [`Language::slug`]. Variants are gated by the same `lang-*` cargo
        /// features as the grammar lookup table, so each build exposes the
        /// variants enabled for that build.
        ///
        /// ```rust
        /// use dioxus_code::Language;
        /// assert_eq!(Language::Rust.slug(), "rust");
        /// assert_eq!(Language::from_slug("brainfuck"), None);
        /// ```
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        #[non_exhaustive]
        pub enum Language {
            /// Automatically detect the language from the source text.
            #[cfg(feature = "detection")]
            Auto,
            $(
                $(#[$attr])*
                #[doc = concat!("Arborium slug `\"", $slug, "\"`.")]
                $variant,
            )*
        }

        impl Language {
            /// Every variant compiled into this build, in declaration order.
            ///
            /// Variants whose grammars are gated behind cargo features only
            /// appear here when the corresponding feature is enabled.
            ///
            /// ```rust
            /// use dioxus_code::Language;
            /// assert!(Language::ALL.contains(&Language::Rust));
            /// ```
            pub const ALL: &'static [Language] = &[
                #[cfg(feature = "detection")]
                Self::Auto,
                $(
                    $(#[$attr])*
                    Self::$variant,
                )*
            ];

            /// Arborium slug for this language.
            pub const fn slug(self) -> &'static str {
                match self {
                    #[cfg(feature = "detection")]
                    Self::Auto => "auto",
                    $(
                        $(#[$attr])*
                        Self::$variant => $slug,
                    )*
                }
            }

            /// Parse an Arborium slug into a [`Language`].
            ///
            /// Returns `None` for unknown slugs and for slugs whose grammar
            /// feature is disabled in this build.
            pub fn from_slug(slug: &str) -> Option<Self> {
                match slug {
                    $(
                        $(#[$attr])*
                        $slug => Some(Self::$variant),
                    )*
                    _ => None,
                }
            }
        }
    };
}

impl Language {
    /// Best-effort detection from a path or filename.
    ///
    /// Wraps [`arborium::detect_language`] and maps the resulting extension into a
    /// [`Language`] variant, returning `None` when detection fails or the
    /// detected language's grammar feature is disabled in this build.
    /// With the `detection` feature enabled, use `Language::detect_source` to
    /// detect from source contents.
    ///
    /// Available with the `runtime` feature.
    #[cfg(feature = "runtime")]
    #[cfg_attr(docsrs, doc(cfg(feature = "runtime")))]
    pub fn detect(input: &str) -> Option<Self> {
        arborium::detect_language(input).and_then(Self::from_slug)
    }

    /// Best-effort detection from source contents.
    ///
    /// Wraps [`betlang::detect`] and maps the detected language enum directly
    /// into a [`Language`] variant, returning `None` when detection fails or
    /// the detected language's grammar feature is disabled in this build.
    ///
    /// Available with the `detection` feature.
    ///
    /// ```rust
    /// use dioxus_code::Language;
    ///
    /// assert_eq!(
    ///     Language::detect_source("use std::fmt;\nfn main() { println!(\"hi\"); }"),
    ///     Some(Language::Rust),
    /// );
    /// ```
    #[cfg(feature = "detection")]
    #[cfg_attr(docsrs, doc(cfg(feature = "detection")))]
    pub fn detect_source(source: &str) -> Option<Self> {
        betlang::detect(source)
            .language()
            .and_then(Self::from_betlang_language)
    }

    #[cfg(feature = "detection")]
    fn from_betlang_language(language: betlang::Language) -> Option<Self> {
        match language {
            #[cfg(feature = "lang-asm")]
            betlang::Language::Asm => Some(Self::Asm),
            #[cfg(feature = "lang-batch")]
            betlang::Language::Batch => Some(Self::Batch),
            #[cfg(feature = "lang-c")]
            betlang::Language::C => Some(Self::C),
            #[cfg(feature = "lang-clojure")]
            betlang::Language::Clojure => Some(Self::Clojure),
            #[cfg(feature = "lang-cmake")]
            betlang::Language::CMake => Some(Self::CMake),
            #[cfg(feature = "lang-cobol")]
            betlang::Language::Cobol => Some(Self::Cobol),
            #[cfg(feature = "lang-cpp")]
            betlang::Language::Cpp => Some(Self::Cpp),
            #[cfg(feature = "lang-c-sharp")]
            betlang::Language::Cs => Some(Self::CSharp),
            #[cfg(feature = "lang-css")]
            betlang::Language::Css => Some(Self::Css),
            #[cfg(feature = "lang-dart")]
            betlang::Language::Dart => Some(Self::Dart),
            #[cfg(feature = "lang-dockerfile")]
            betlang::Language::Dockerfile => Some(Self::Dockerfile),
            #[cfg(feature = "lang-elixir")]
            betlang::Language::Elixir => Some(Self::Elixir),
            #[cfg(feature = "lang-erlang")]
            betlang::Language::Erlang => Some(Self::Erlang),
            #[cfg(feature = "lang-ruby")]
            betlang::Language::Gemfile | betlang::Language::Gemspec | betlang::Language::Ruby => {
                Some(Self::Ruby)
            }
            #[cfg(feature = "lang-go")]
            betlang::Language::Go => Some(Self::Go),
            #[cfg(feature = "lang-groovy")]
            betlang::Language::Gradle | betlang::Language::Groovy => Some(Self::Groovy),
            #[cfg(feature = "lang-haskell")]
            betlang::Language::Haskell => Some(Self::Haskell),
            #[cfg(feature = "lang-html")]
            betlang::Language::Html => Some(Self::Html),
            #[cfg(feature = "lang-ini")]
            betlang::Language::Ini => Some(Self::Ini),
            #[cfg(feature = "lang-java")]
            betlang::Language::Java => Some(Self::Java),
            #[cfg(feature = "lang-javascript")]
            betlang::Language::JavaScript => Some(Self::JavaScript),
            #[cfg(feature = "lang-json")]
            betlang::Language::Json => Some(Self::Json),
            #[cfg(feature = "lang-julia")]
            betlang::Language::Julia => Some(Self::Julia),
            #[cfg(feature = "lang-kotlin")]
            betlang::Language::Kotlin => Some(Self::Kotlin),
            #[cfg(feature = "lang-commonlisp")]
            betlang::Language::Lisp => Some(Self::CommonLisp),
            #[cfg(feature = "lang-lua")]
            betlang::Language::Lua => Some(Self::Lua),
            #[cfg(feature = "lang-markdown")]
            betlang::Language::Markdown => Some(Self::Markdown),
            #[cfg(feature = "lang-objc")]
            betlang::Language::ObjectiveC => Some(Self::ObjectiveC),
            #[cfg(feature = "lang-ocaml")]
            betlang::Language::Ocaml => Some(Self::OCaml),
            #[cfg(feature = "lang-perl")]
            betlang::Language::Perl => Some(Self::Perl),
            #[cfg(feature = "lang-php")]
            betlang::Language::Php => Some(Self::Php),
            #[cfg(feature = "lang-powershell")]
            betlang::Language::Powershell => Some(Self::PowerShell),
            #[cfg(feature = "lang-python")]
            betlang::Language::Python => Some(Self::Python),
            #[cfg(feature = "lang-r")]
            betlang::Language::R => Some(Self::R),
            betlang::Language::Rust => Some(Self::Rust),
            #[cfg(feature = "lang-scala")]
            betlang::Language::Scala => Some(Self::Scala),
            #[cfg(feature = "lang-bash")]
            betlang::Language::Shell => Some(Self::Bash),
            #[cfg(feature = "lang-sql")]
            betlang::Language::Sql => Some(Self::Sql),
            #[cfg(feature = "lang-swift")]
            betlang::Language::Swift => Some(Self::Swift),
            #[cfg(feature = "lang-toml")]
            betlang::Language::Toml => Some(Self::Toml),
            #[cfg(feature = "lang-typescript")]
            betlang::Language::TypeScript => Some(Self::TypeScript),
            #[cfg(feature = "lang-vb")]
            betlang::Language::Vba => Some(Self::VisualBasic),
            #[cfg(feature = "lang-verilog")]
            betlang::Language::Verilog => Some(Self::Verilog),
            #[cfg(feature = "lang-xml")]
            betlang::Language::Xml => Some(Self::Xml),
            #[cfg(feature = "lang-yaml")]
            betlang::Language::Yaml => Some(Self::Yaml),
            _ => None,
        }
    }
}

define_languages! {
    Rust => "rust",
    #[cfg(feature = "lang-ada")]
    Ada => "ada",
    #[cfg(feature = "lang-agda")]
    Agda => "agda",
    #[cfg(feature = "lang-asciidoc")]
    Asciidoc => "asciidoc",
    #[cfg(feature = "lang-asm")]
    Asm => "asm",
    #[cfg(feature = "lang-awk")]
    Awk => "awk",
    #[cfg(feature = "lang-bash")]
    Bash => "bash",
    #[cfg(feature = "lang-batch")]
    Batch => "batch",
    #[cfg(feature = "lang-c")]
    C => "c",
    #[cfg(feature = "lang-c-sharp")]
    CSharp => "c-sharp",
    #[cfg(feature = "lang-caddy")]
    Caddy => "caddy",
    #[cfg(feature = "lang-capnp")]
    Capnp => "capnp",
    #[cfg(feature = "lang-cedar")]
    Cedar => "cedar",
    #[cfg(feature = "lang-cedarschema")]
    CedarSchema => "cedarschema",
    #[cfg(feature = "lang-clojure")]
    Clojure => "clojure",
    #[cfg(feature = "lang-cmake")]
    CMake => "cmake",
    #[cfg(feature = "lang-cobol")]
    Cobol => "cobol",
    #[cfg(feature = "lang-commonlisp")]
    CommonLisp => "commonlisp",
    #[cfg(feature = "lang-cpp")]
    Cpp => "cpp",
    #[cfg(feature = "lang-css")]
    Css => "css",
    #[cfg(feature = "lang-d")]
    D => "d",
    #[cfg(feature = "lang-dart")]
    Dart => "dart",
    #[cfg(feature = "lang-devicetree")]
    DeviceTree => "devicetree",
    #[cfg(feature = "lang-diff")]
    Diff => "diff",
    #[cfg(feature = "lang-dockerfile")]
    Dockerfile => "dockerfile",
    #[cfg(feature = "lang-dot")]
    Dot => "dot",
    #[cfg(feature = "lang-elisp")]
    Elisp => "elisp",
    #[cfg(feature = "lang-elixir")]
    Elixir => "elixir",
    #[cfg(feature = "lang-elm")]
    Elm => "elm",
    #[cfg(feature = "lang-erlang")]
    Erlang => "erlang",
    #[cfg(feature = "lang-fish")]
    Fish => "fish",
    #[cfg(feature = "lang-fsharp")]
    FSharp => "fsharp",
    #[cfg(feature = "lang-gleam")]
    Gleam => "gleam",
    #[cfg(feature = "lang-glsl")]
    Glsl => "glsl",
    #[cfg(feature = "lang-go")]
    Go => "go",
    #[cfg(feature = "lang-graphql")]
    GraphQL => "graphql",
    #[cfg(feature = "lang-groovy")]
    Groovy => "groovy",
    #[cfg(feature = "lang-haskell")]
    Haskell => "haskell",
    #[cfg(feature = "lang-hcl")]
    Hcl => "hcl",
    #[cfg(feature = "lang-hlsl")]
    Hlsl => "hlsl",
    #[cfg(feature = "lang-html")]
    Html => "html",
    #[cfg(feature = "lang-idris")]
    Idris => "idris",
    #[cfg(feature = "lang-ini")]
    Ini => "ini",
    #[cfg(feature = "lang-java")]
    Java => "java",
    #[cfg(feature = "lang-javascript")]
    JavaScript => "javascript",
    #[cfg(feature = "lang-jinja2")]
    Jinja2 => "jinja2",
    #[cfg(feature = "lang-jq")]
    Jq => "jq",
    #[cfg(feature = "lang-json")]
    Json => "json",
    #[cfg(feature = "lang-julia")]
    Julia => "julia",
    #[cfg(feature = "lang-kotlin")]
    Kotlin => "kotlin",
    #[cfg(feature = "lang-lean")]
    Lean => "lean",
    #[cfg(feature = "lang-lua")]
    Lua => "lua",
    #[cfg(feature = "lang-markdown")]
    Markdown => "markdown",
    #[cfg(feature = "lang-matlab")]
    Matlab => "matlab",
    #[cfg(feature = "lang-meson")]
    Meson => "meson",
    #[cfg(feature = "lang-nginx")]
    Nginx => "nginx",
    #[cfg(feature = "lang-ninja")]
    Ninja => "ninja",
    #[cfg(feature = "lang-nix")]
    Nix => "nix",
    #[cfg(feature = "lang-objc")]
    ObjectiveC => "objc",
    #[cfg(feature = "lang-ocaml")]
    OCaml => "ocaml",
    #[cfg(feature = "lang-perl")]
    Perl => "perl",
    #[cfg(feature = "lang-php")]
    Php => "php",
    #[cfg(feature = "lang-postscript")]
    PostScript => "postscript",
    #[cfg(feature = "lang-powershell")]
    PowerShell => "powershell",
    #[cfg(feature = "lang-prolog")]
    Prolog => "prolog",
    #[cfg(feature = "lang-python")]
    Python => "python",
    #[cfg(feature = "lang-query")]
    Query => "query",
    #[cfg(feature = "lang-r")]
    R => "r",
    #[cfg(feature = "lang-rego")]
    Rego => "rego",
    #[cfg(feature = "lang-rescript")]
    Rescript => "rescript",
    #[cfg(feature = "lang-ron")]
    Ron => "ron",
    #[cfg(feature = "lang-ruby")]
    Ruby => "ruby",
    #[cfg(feature = "lang-scala")]
    Scala => "scala",
    #[cfg(feature = "lang-scheme")]
    Scheme => "scheme",
    #[cfg(feature = "lang-scss")]
    Scss => "scss",
    #[cfg(feature = "lang-solidity")]
    Solidity => "solidity",
    #[cfg(feature = "lang-sparql")]
    Sparql => "sparql",
    #[cfg(feature = "lang-sql")]
    Sql => "sql",
    #[cfg(feature = "lang-ssh-config")]
    SshConfig => "ssh-config",
    #[cfg(feature = "lang-starlark")]
    Starlark => "starlark",
    #[cfg(feature = "lang-styx")]
    Styx => "styx",
    #[cfg(feature = "lang-svelte")]
    Svelte => "svelte",
    #[cfg(feature = "lang-swift")]
    Swift => "swift",
    #[cfg(feature = "lang-textproto")]
    Textproto => "textproto",
    #[cfg(feature = "lang-thrift")]
    Thrift => "thrift",
    #[cfg(feature = "lang-tlaplus")]
    TlaPlus => "tlaplus",
    #[cfg(feature = "lang-toml")]
    Toml => "toml",
    #[cfg(feature = "lang-tsx")]
    Tsx => "tsx",
    #[cfg(feature = "lang-typescript")]
    TypeScript => "typescript",
    #[cfg(feature = "lang-typst")]
    Typst => "typst",
    #[cfg(feature = "lang-uiua")]
    Uiua => "uiua",
    #[cfg(feature = "lang-vb")]
    VisualBasic => "vb",
    #[cfg(feature = "lang-verilog")]
    Verilog => "verilog",
    #[cfg(feature = "lang-vhdl")]
    Vhdl => "vhdl",
    #[cfg(feature = "lang-vim")]
    Vim => "vim",
    #[cfg(feature = "lang-vue")]
    Vue => "vue",
    #[cfg(feature = "lang-wit")]
    Wit => "wit",
    #[cfg(feature = "lang-x86asm")]
    X86Asm => "x86asm",
    #[cfg(feature = "lang-xml")]
    Xml => "xml",
    #[cfg(feature = "lang-yaml")]
    Yaml => "yaml",
    #[cfg(feature = "lang-yuri")]
    Yuri => "yuri",
    #[cfg(feature = "lang-zig")]
    Zig => "zig",
    #[cfg(feature = "lang-zsh")]
    Zsh => "zsh",
}

#[cfg(all(test, feature = "detection"))]
mod tests {
    use super::*;

    macro_rules! assert_betlang_mapping {
        ($betlang:expr, $feature:literal, $language:expr) => {
            assert_eq!(Language::from_betlang_language($betlang), {
                #[cfg(feature = $feature)]
                {
                    Some($language)
                }
                #[cfg(not(feature = $feature))]
                {
                    None
                }
            });
        };
    }

    #[test]
    fn maps_betlang_languages_directly() {
        assert_eq!(
            Language::from_betlang_language(betlang::Language::Rust),
            Some(Language::Rust),
        );

        assert_betlang_mapping!(betlang::Language::Cs, "lang-c-sharp", Language::CSharp);
        assert_betlang_mapping!(
            betlang::Language::Lisp,
            "lang-commonlisp",
            Language::CommonLisp
        );
        assert_betlang_mapping!(
            betlang::Language::ObjectiveC,
            "lang-objc",
            Language::ObjectiveC
        );
        assert_betlang_mapping!(betlang::Language::Shell, "lang-bash", Language::Bash);
        assert_betlang_mapping!(betlang::Language::Vba, "lang-vb", Language::VisualBasic);
        assert_betlang_mapping!(betlang::Language::Gemfile, "lang-ruby", Language::Ruby);
        assert_betlang_mapping!(betlang::Language::Gemspec, "lang-ruby", Language::Ruby);
        assert_betlang_mapping!(betlang::Language::Gradle, "lang-groovy", Language::Groovy);
    }

    #[test]
    fn detect_does_not_fall_back_to_source_contents() {
        let source = "use std::fmt;\nfn main() { println!(\"hi\"); }";

        assert_eq!(Language::detect(source), None);
        assert_eq!(Language::detect_source(source), Some(Language::Rust));
    }
}
