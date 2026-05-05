<div align="center">
  <h1>dioxus-code-editor</h1>
  <p><strong>Syntax-highlighted code editor component for Dioxus.</strong></p>
</div>

<div align="center">
  <a href="https://crates.io/crates/dioxus-code-editor">
    <img src="https://img.shields.io/crates/v/dioxus-code-editor.svg?style=flat-square" alt="Crates.io version" />
  </a>
  <a href="https://crates.io/crates/dioxus-code-editor">
    <img src="https://img.shields.io/crates/d/dioxus-code-editor.svg?style=flat-square" alt="Downloads" />
  </a>
  <a href="https://docs.rs/dioxus-code-editor">
    <img src="https://img.shields.io/badge/docs-latest-blue.svg?style=flat-square" alt="docs.rs" />
  </a>
</div>

---

A controlled, syntax-highlighted code editor for Dioxus. Pairs a `contenteditable` input layer with the [`dioxus-code`](https://crates.io/crates/dioxus-code) highlighter so user edits stay in sync with rendered tokens.

## Quick start

```toml
[dependencies]
dioxus-code-editor = "0.0.1"
```

```rust
use dioxus::prelude::*;
use dioxus_code::Theme;
use dioxus_code_editor::CodeEditor;

#[component]
fn Editor() -> Element {
    let mut source = use_signal(String::new);

    rsx! {
        CodeEditor {
            value: source(),
            language: "rust",
            theme: Theme::TOKYO_NIGHT,
            oninput: move |value| source.set(value),
        }
    }
}
```

The component is controlled — drive `value` from your own signal and update it inside `oninput`.

## Props

| prop | description |
|---|---|
| `value` | Current editor contents. |
| `language` | Arborium language hint, e.g. `"rust"`. |
| `name` | Filename used for language detection when `language` is empty. |
| `theme` | Syntax theme shared with `dioxus-code`. |
| `line_numbers` | Show a one-based line gutter. Defaults to `true`. |
| `read_only` | Disable editing while preserving highlighting. |
| `spellcheck` | Forward `spellcheck` to the input layer. |
| `aria_label` | Accessible label for the editor textbox. |
| `placeholder` | Shown only while `value` is empty. |
| `class` | Extra class names appended to the editor root. |
| `oninput` | Called with the full editor text after each input event. |

## License

[MIT](LICENSE).
