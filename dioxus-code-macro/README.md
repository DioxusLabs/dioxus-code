<div align="center">
  <h1>dioxus-code-macro</h1>
  <p><strong>Compile-time syntax highlighting macro for <a href="https://crates.io/crates/dioxus-code">dioxus-code</a>.</strong></p>
</div>

<div align="center">
  <a href="https://crates.io/crates/dioxus-code-macro">
    <img src="https://img.shields.io/crates/v/dioxus-code-macro.svg?style=flat-square" alt="Crates.io version" />
  </a>
  <a href="https://crates.io/crates/dioxus-code-macro">
    <img src="https://img.shields.io/crates/d/dioxus-code-macro.svg?style=flat-square" alt="Downloads" />
  </a>
  <a href="https://docs.rs/dioxus-code-macro">
    <img src="https://img.shields.io/badge/docs-latest-blue.svg?style=flat-square" alt="docs.rs" />
  </a>
</div>

---

Implementation crate for the [`code!`] macro re-exported by [`dioxus-code`](https://crates.io/crates/dioxus-code) under its default `macro` feature. You usually depend on `dioxus-code` instead of pulling this in directly.

The macro reads a source file at compile time, parses it with [`arborium`](https://crates.io/crates/arborium), and expands to a static span tree. The runtime binary ships only the spans — no parser.

```rust
use dioxus_code::code;

let _tree = code!("/snippets/demo.rs");
```

## Path resolution

Paths are resolved relative to the *consumer's* `CARGO_MANIFEST_DIR`. A leading `/` anchors to that directory; bare paths resolve from it as well. `concat!(...)` and `env!(...)` expressions are also accepted.

```rust
# use dioxus_code::code;
let _tree = code!(concat!(env!("CARGO_MANIFEST_DIR"), "/snippets/demo.rs"));
```

## Explicit language

When the file extension isn't enough to infer the language, pass
[`CodeOptions::builder`] with [`CodeOptions::with_language`]:

```rust
use dioxus_code::{CodeOptions, Language, code};

let _tree = code!(
    "/snippets/demo.rs",
    CodeOptions::builder().with_language(Language::Rust)
);
```

## License

MIT.

[`code!`]: https://docs.rs/dioxus-code-macro/latest/dioxus_code_macro/macro.code.html
[`CodeOptions::builder`]: https://docs.rs/dioxus-code/latest/dioxus_code/struct.CodeOptions.html#method.builder
[`CodeOptions::with_language`]: https://docs.rs/dioxus-code/latest/dioxus_code/struct.CodeOptions.html#method.with_language
