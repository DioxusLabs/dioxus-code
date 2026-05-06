<div align="center">
  <h1>dioxus-code</h1>
  <p><strong>Syntax-highlighted code blocks for Dioxus.</strong></p>
</div>

<div align="center">
  <a href="https://crates.io/crates/dioxus-code">
    <img src="https://img.shields.io/crates/v/dioxus-code.svg?style=flat-square" alt="Crates.io version" />
  </a>
  <a href="https://crates.io/crates/dioxus-code">
    <img src="https://img.shields.io/crates/d/dioxus-code.svg?style=flat-square" alt="Downloads" />
  </a>
  <a href="https://docs.rs/dioxus-code">
    <img src="https://img.shields.io/badge/docs-latest-blue.svg?style=flat-square" alt="docs.rs" />
  </a>
</div>

---

A small Dioxus component for rendering source code with proper highlighting. Parsing is powered by [arborium](https://crates.io/crates/arborium); themes ship as scoped CSS so you can mix several on one page.

Two ways to highlight:

- **`code!()` macro** — parses at compile time. The runtime ships only the spans, no parser. Default.
- **`SourceCode`** — parses at runtime. Opt in with the `runtime` feature when the source isn't known until the user types it.

## Quick start

```toml
[dependencies]
dioxus-code = "0.1"
```

```rust,ignore
use dioxus::prelude::*;
use dioxus_code::{Code, Theme, code};

#[component]
fn ReadMe() -> Element {
    rsx! {
        Code {
            src: code!("/snippets/demo.rs"),
            theme: Theme::TOKYO_NIGHT,
        }
    }
}
```

The path is resolved from the consumer's `CARGO_MANIFEST_DIR`. `concat!` and `env!` work too.

## Runtime highlighting

For editor-style use cases where the source isn't known at compile time:

```toml
[dependencies]
dioxus-code = { version = "0.1", features = ["runtime"] }
```

```rust,ignore
use dioxus_code::{Code, SourceCode, Theme};

rsx! {
    Code {
        src: SourceCode::new(user_input).with_language("rust"),
        theme: Theme::GITHUB_LIGHT,
    }
}
```

Language can be set explicitly, inferred from a filename via `with_name("main.rs")`, or auto-detected from the source. The default `runtime` feature includes Rust; pass `lang-python`, `lang-toml`, or `all-languages` for the rest.

## Editor

`dioxus-code-editor` is a sibling crate that pairs the highlighter with a `contenteditable` input layer:

```rust,ignore
use dioxus_code_editor::CodeEditor;
use dioxus_code::Theme;

let mut source = use_signal(|| String::new());

rsx! {
    CodeEditor {
        value: source(),
        language: "rust",
        theme: Theme::TOKYO_NIGHT,
        oninput: move |value| source.set(value),
    }
}
```

It is controlled — drive `value` from your own signal and update it inside `oninput`.

## Themes

Thirty-odd built-ins, including Tokyo Night, Catppuccin (all four), Dracula, GitHub Light/Dark, Gruvbox, Nord, One Dark, Rosé Pine, Solarized, the Rustdoc themes, and others. Each is exposed as a `Theme` constant and a CSS asset; pages with multiple themes render side-by-side without leaking styles.

```rust,ignore
Code { src: code!("/example.rs"), theme: Theme::CATPPUCCIN_MOCHA }
```

Use `CodeTheme::system` to select a light and dark theme with CSS media
queries. This is JavaScript-free and works during SSR:

```rust,ignore
use dioxus_code::{Code, CodeTheme, Theme, code};

Code {
    src: code!("/example.rs"),
    theme: CodeTheme::system(Theme::GITHUB_LIGHT, Theme::TOKYO_NIGHT),
}
```

## Examples

```sh
dx serve --example dioxus-code-basic       # macro + runtime side by side
dx serve --example dioxus-code-macro-only  # compile-time only, no parser in the binary
dx serve --example dioxus-code-live-input  # textarea bound to runtime highlighter
```

## Workspace layout

| crate | purpose |
|---|---|
| `dioxus-code` | The `Code` component, themes, and runtime/macro entry points. |
| `dioxus-code-editor` | Editable code surface built on `Code`. |
| `dioxus-code-macro` | Implementation of `code!()`. Re-exported by `dioxus-code` under the `macro` feature. |

## License

MIT. See the repository `LICENSE` file.
