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

A controlled, syntax-highlighted code editor for Dioxus. Pairs an editable input layer with the [`dioxus-code`](https://crates.io/crates/dioxus-code) highlighter so user edits stay in sync with rendered tokens.

## Quick start

```toml
[dependencies]
dioxus-code-editor = "0.1.2"
```

```rust
use dioxus::prelude::*;
use dioxus_code::Theme;
use dioxus_code_editor::{CodeEditor, Language};

#[component]
fn Editor() -> Element {
    let mut source = use_signal(String::new);

    rsx! {
        CodeEditor {
            value: source(),
            language: Language::Rust,
            theme: Theme::TOKYO_NIGHT,
            oninput: move |value| source.set(value),
        }
    }
}
```

The component is controlled — drive [`CodeEditorProps::value`] from your own signal and update it inside [`CodeEditorProps::oninput`].

## Props

| prop | description |
|---|---|
| [`CodeEditorProps::value`] | Current editor contents. |
| [`CodeEditorProps::language`] | Syntax grammar selection. Pass a [`Language`] variant (for example [`Language::Rust`]) or use [`Language::from_slug`] for runtime slugs. |
| [`CodeEditorProps::theme`] | Syntax theme selection shared with [`dioxus-code`](https://crates.io/crates/dioxus-code); accepts [`Theme`] or [`CodeTheme`]. |
| [`CodeEditorProps::line_numbers`] | Show a one-based line gutter. Defaults to `true`. |
| [`CodeEditorProps::read_only`] | Disable editing while preserving highlighting. |
| [`CodeEditorProps::spellcheck`] | Forward `spellcheck` to the input layer. |
| [`CodeEditorProps::aria_label`] | Accessible label for the editor textbox. |
| [`CodeEditorProps::placeholder`] | Shown only while [`CodeEditorProps::value`] is empty. |
| [`CodeEditorProps::class`] | Extra class names appended to the editor root. |
| [`CodeEditorProps::oninput`] | Called with the full editor text after each input event. |

## License

MIT.

[`CodeEditorProps::aria_label`]: https://docs.rs/dioxus-code-editor/latest/dioxus_code_editor/struct.CodeEditorProps.html#structfield.aria_label
[`CodeEditorProps::class`]: https://docs.rs/dioxus-code-editor/latest/dioxus_code_editor/struct.CodeEditorProps.html#structfield.class
[`CodeEditorProps::language`]: https://docs.rs/dioxus-code-editor/latest/dioxus_code_editor/struct.CodeEditorProps.html#structfield.language
[`CodeEditorProps::line_numbers`]: https://docs.rs/dioxus-code-editor/latest/dioxus_code_editor/struct.CodeEditorProps.html#structfield.line_numbers
[`CodeEditorProps::oninput`]: https://docs.rs/dioxus-code-editor/latest/dioxus_code_editor/struct.CodeEditorProps.html#structfield.oninput
[`CodeEditorProps::placeholder`]: https://docs.rs/dioxus-code-editor/latest/dioxus_code_editor/struct.CodeEditorProps.html#structfield.placeholder
[`CodeEditorProps::read_only`]: https://docs.rs/dioxus-code-editor/latest/dioxus_code_editor/struct.CodeEditorProps.html#structfield.read_only
[`CodeEditorProps::spellcheck`]: https://docs.rs/dioxus-code-editor/latest/dioxus_code_editor/struct.CodeEditorProps.html#structfield.spellcheck
[`CodeEditorProps::theme`]: https://docs.rs/dioxus-code-editor/latest/dioxus_code_editor/struct.CodeEditorProps.html#structfield.theme
[`CodeEditorProps::value`]: https://docs.rs/dioxus-code-editor/latest/dioxus_code_editor/struct.CodeEditorProps.html#structfield.value
[`CodeTheme`]: https://docs.rs/dioxus-code/latest/dioxus_code/struct.CodeTheme.html
[`Language`]: https://docs.rs/dioxus-code/latest/dioxus_code/enum.Language.html
[`Language::from_slug`]: https://docs.rs/dioxus-code/latest/dioxus_code/enum.Language.html#method.from_slug
[`Language::Rust`]: https://docs.rs/dioxus-code/latest/dioxus_code/enum.Language.html#variant.Rust
[`Theme`]: https://docs.rs/dioxus-code/latest/dioxus_code/struct.Theme.html
