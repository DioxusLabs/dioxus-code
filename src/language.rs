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
    /// Best-effort detection from a path, filename, shebang, or file contents.
    ///
    /// Wraps [`arborium::detect_language`] and maps the resulting slug into a
    /// [`Language`] variant, returning `None` when detection fails or the
    /// detected language's grammar feature is disabled in this build.
    ///
    /// Available with the `runtime` feature.
    #[cfg(feature = "runtime")]
    #[cfg_attr(docsrs, doc(cfg(feature = "runtime")))]
    pub fn detect(input: &str) -> Option<Self> {
        let detected = arborium::detect_language(input).and_then(Self::from_slug);

        #[cfg(feature = "detection")]
        {
            detected.or_else(|| {
                betlang::detect(input)
                    .language()
                    .and_then(|language| Self::from_slug(language.slug()))
            })
        }

        #[cfg(not(feature = "detection"))]
        {
            detected
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
