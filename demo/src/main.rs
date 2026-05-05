use dioxus::prelude::*;
use dioxus_code::{Code, RuntimeCode, Theme, code};

const STARTER: &str = r#"use dioxus::prelude::*;

#[component]
fn Counter() -> Element {
    let mut count = use_signal(|| 0);

    rsx! {
        button {
            onclick: move |_| count += 1,
            "Count: {count}"
        }
    }
}
"#;

const PYTHON: &str = r#"def normalize(values):
    total = sum(values)
    return [round(value / total, 3) for value in values]

weights = normalize([12, 18, 7, 31])
print(weights)
"#;

const DOCS_INSTALL: &str = r#"dioxus-code = { version = "0.1", features = ["runtime"] }
"#;

const DOCS_RUNTIME: &str = r#"use dioxus_code::{Code, RuntimeCode, Theme};

rsx! {
    Code {
        src: RuntimeCode::new(source).with_language("rust"),
        theme: Theme::TOKYO_NIGHT,
    }
}
"#;

const DOCS_STATIC: &str = r#"use dioxus_code::{Code, Theme, code};

rsx! {
    Code {
        src: code!("/snippets/example.rs"),
        theme: Theme::RUSTDOC_AYU,
    }
}
"#;

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    let source = use_signal(|| STARTER.to_string());
    let active_theme = use_signal(|| 0usize);

    rsx! {
        style { {APP_CSS} }
        main { class: "site-shell",
            Header {}
            Hero { source: source(), theme: demo_themes()[active_theme()].theme }
            Playground { source, active_theme }
            Demos {}
            Docs {}
            SiteFooter {}
        }
    }
}

#[component]
fn Header() -> Element {
    rsx! {
        header { class: "topbar",
            a { class: "brand", href: "#top", "aria-label": "Homepage",
                span { class: "brand-mark", "dx" }
                span { "dioxus-code" }
            }
            nav {
                a { href: "#playground", "Playground" }
                a { href: "#demos", "Demos" }
                a { href: "#docs", "Docs" }
                a { class: "topbar-cta", href: "https://crates.io/crates/dioxus-code", "crates.io ↗" }
            }
        }
    }
}

#[component]
fn Hero(source: String, theme: Theme) -> Element {
    rsx! {
        section { id: "top", class: "section hero",
            div { class: "hero-grid",
                div { class: "card card-pitch",
                    span { class: "card-eyebrow card-eyebrow-light", "dioxus-code · v0.1" }
                    h1 { class: "pitch-title",
                        "The code block, redesigned for Dioxus apps."
                    }
                    p { class: "pitch-lede",
                        "Drop-in component, two source modes, themes you'd actually choose."
                    }
                    div { class: "pitch-actions",
                        a { class: "cta primary", href: "#playground", "Try it →" }
                        a { class: "cta", href: "#docs", "Documentation" }
                    }
                    div { class: "pitch-meta",
                        span { class: "pitch-meta-dot" }
                        span { "Built on Arborium · works with any Tree-sitter grammar" }
                    }
                }
                div { class: "card card-code",
                    div { class: "card-bar",
                        span { "live preview" }
                        span { "{theme.name()}" }
                    }
                    div { class: "card-code-body",
                        Code {
                            src: RuntimeCode::new(source).with_language("rust"),
                            theme,
                        }
                    }
                }
                div { class: "card card-install",
                    span { class: "card-eyebrow", "Install" }
                    code { class: "shell-cmd", "cargo add dioxus-code" }
                    p { class: "card-note", "Add the runtime feature for user-supplied source." }
                }
                div { class: "card card-modes",
                    span { class: "card-eyebrow", "Two source modes" }
                    div { class: "modes-row",
                        div { class: "mode-cell",
                            p { class: "mode-name", "code!()" }
                            span { class: "mode-desc", "compile-time macro" }
                            span { class: "mode-tag", "0kb runtime" }
                        }
                        div { class: "mode-cell",
                            p { class: "mode-name", "RuntimeCode" }
                            span { class: "mode-desc", "user input · network · generated" }
                            span { class: "mode-tag", "tree-sitter detection" }
                        }
                    }
                }
                div { class: "card card-zero",
                    span { class: "card-eyebrow", "Compile-time output" }
                    p { class: "stat-num", "0kb" }
                    p { class: "card-note",
                        "Static snippets ship as pre-tokenized markup. No JS, no runtime parser, no flash of unstyled code."
                    }
                }
                div { class: "card card-themes",
                    span { class: "card-eyebrow", "32 themes shipped" }
                    div { class: "swatches",
                        for swatch in demo_themes() {
                            span { class: "swatch-chip", style: "background:{swatch.accent};" }
                        }
                        span { class: "swatch-more", "+27" }
                    }
                    p { class: "card-note",
                        "Tokyo Night, GitHub, Catppuccin, Gruvbox, Rose Pine, Ayu, Solarized — pick one or expose all of them."
                    }
                }
            }
        }
    }
}

#[component]
fn Playground(mut source: Signal<String>, mut active_theme: Signal<usize>) -> Element {
    let theme = demo_themes()[active_theme()].theme;
    let active_idx = active_theme();
    let active_swatch = demo_themes()[active_idx].accent;
    rsx! {
        section { id: "playground", class: "section",
            div { class: "section-head",
                div {
                    span { class: "section-eyebrow", "// Live playground" }
                    h2 { class: "section-title", "Edit on the left. Render on the right." }
                }
                p { class: "section-sub",
                    "Type Rust, swap themes, and ship the same component to your users."
                }
            }
            div { class: "playground-grid",
                div { class: "card card-editor",
                    div { class: "card-bar",
                        span { "source.rs" }
                        span { "rust · " {format!("{} chars", source().chars().count())} }
                    }
                    textarea {
                        class: "code-input",
                        spellcheck: false,
                        value: "{source}",
                        oninput: move |event| source.set(event.value()),
                    }
                }
                div { class: "card card-preview",
                    div { class: "card-bar",
                        span { "preview" }
                        span { class: "preview-meta",
                            span { class: "preview-swatch", style: "background:{active_swatch};" }
                            "{theme.name()}"
                        }
                    }
                    div { class: "card-code-body",
                        Code {
                            src: RuntimeCode::new(source()).with_language("rust"),
                            theme,
                        }
                    }
                }
                div { class: "card card-themepicker",
                    div { class: "card-bar",
                        span { "active theme" }
                        span { {format!("{} of {}", active_idx + 1, demo_themes().len())} }
                    }
                    div { class: "theme-strip",
                        for (index, swatch) in demo_themes().iter().enumerate() {
                            button {
                                class: if active_theme() == index { "theme-pill active" } else { "theme-pill" },
                                onclick: move |_| active_theme.set(index),
                                span { class: "theme-pill-swatch", style: "background:{swatch.accent};" }
                                span { "{swatch.theme.name()}" }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn Demos() -> Element {
    rsx! {
        section { id: "demos", class: "section",
            div { class: "section-head",
                div {
                    span { class: "section-eyebrow", "// Three rendering modes" }
                    h2 { class: "section-title", "One component. Real-world examples." }
                }
                p { class: "section-sub",
                    "Static at compile time, runtime detection, fallback for unknown languages — same component, different src input."
                }
            }
            div { class: "demos-grid",
                article { class: "card card-demo card-demo-feature",
                    div { class: "demo-feature-head",
                        span { class: "card-eyebrow", "Compile time · code! macro" }
                        span { class: "demo-feature-tag", "0kb runtime" }
                    }
                    h3 { class: "demo-feature-title",
                        "Static snippets, tokenized at build."
                    }
                    p { class: "demo-feature-copy",
                        "Point the macro at a file in your repo. Highlighting happens during cargo build, so the output is plain pre-styled markup. No runtime parser shipped to users."
                    }
                    div { class: "demo-feature-frame",
                        div { class: "card-bar",
                            span { "snippets/palette.rs" }
                            span { "kanagawa-dragon" }
                        }
                        div { class: "card-code-body",
                            Code {
                                src: code!("/snippets/palette.rs"),
                                theme: Theme::KANAGAWA_DRAGON,
                            }
                        }
                    }
                }
                article { class: "card card-demo",
                    div { class: "card-bar",
                        span { "runtime · python" }
                        span { "github-light" }
                    }
                    div { class: "card-code-body",
                        Code {
                            src: RuntimeCode::new(PYTHON).with_language("python"),
                            theme: Theme::GITHUB_LIGHT,
                        }
                    }
                    div { class: "demo-foot",
                        p { class: "card-note",
                            "Pass any string with a known language. Tree-sitter grammars cover Rust, JS, Python, Go, and dozens more."
                        }
                    }
                }
                article { class: "card card-demo",
                    div { class: "card-bar",
                        span { "fallback · plain text" }
                        span { "rustdoc-light" }
                    }
                    div { class: "card-code-body",
                        Code {
                            src: RuntimeCode::new("no language marker here"),
                            theme: Theme::RUSTDOC_LIGHT,
                        }
                    }
                    div { class: "demo-foot",
                        p { class: "card-note",
                            "When the language is unknown, the component renders plain styled text — no fight, no error."
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn Docs() -> Element {
    rsx! {
        section { id: "docs", class: "section",
            div { class: "section-head",
                div {
                    span { class: "section-eyebrow", "// Three steps" }
                    h2 { class: "section-title", "Use it in three moves." }
                }
                p { class: "section-sub",
                    "Add the dependency, pick a theme, and choose between compile-time or runtime source."
                }
            }
            div { class: "docs-grid",
                DocStep {
                    id: "install",
                    num: "01",
                    eyebrow: "Install",
                    title: "Add the dependency",
                    copy: "Enable the runtime feature when source comes from user input, generated files, or network responses.",
                    code: DOCS_INSTALL,
                    language: "toml",
                    theme: Theme::MELANGE_LIGHT,
                }
                DocStep {
                    id: "runtime",
                    num: "02",
                    eyebrow: "Runtime source",
                    title: "RuntimeCode for live input",
                    copy: "Pass any string through RuntimeCode. Provide a language hint when you already know it — Arborium handles tokenizing.",
                    code: DOCS_RUNTIME,
                    language: "rust",
                    theme: Theme::RUSTDOC_AYU,
                }
                DocStep {
                    id: "static",
                    num: "03",
                    eyebrow: "Static source",
                    title: "code! for snippets in your repo",
                    copy: "Use the macro for examples, docs, and any source checked in alongside your app. Highlight markup is generated at compile time.",
                    code: DOCS_STATIC,
                    language: "rust",
                    theme: Theme::TOKYO_NIGHT,
                }
            }
        }
    }
}

#[component]
fn DocStep(
    id: &'static str,
    num: &'static str,
    eyebrow: &'static str,
    title: &'static str,
    copy: &'static str,
    code: &'static str,
    language: &'static str,
    theme: Theme,
) -> Element {
    rsx! {
        article { id, class: "card card-doc",
            div { class: "doc-head",
                span { class: "doc-num", "{num}" }
                span { class: "card-eyebrow", "{eyebrow}" }
            }
            h3 { class: "doc-title", "{title}" }
            p { class: "doc-copy", "{copy}" }
            div { class: "doc-frame",
                Code {
                    src: RuntimeCode::new(code).with_language(language),
                    theme,
                }
            }
        }
    }
}

#[component]
fn SiteFooter() -> Element {
    rsx! {
        footer { class: "section site-footer",
            div { class: "card card-footer",
                div { class: "footer-grid",
                    div { class: "footer-brand",
                        div { class: "footer-brand-row",
                            span { class: "brand-mark", "dx" }
                            span { class: "footer-brand-name", "dioxus-code" }
                        }
                        p { class: "footer-tag",
                            "Syntax highlighting, designed for the inside of your Dioxus app."
                        }
                    }
                    div { class: "footer-col",
                        span { class: "card-eyebrow", "Project" }
                        a { href: "#playground", "Playground" }
                        a { href: "#demos", "Demos" }
                        a { href: "#docs", "Documentation" }
                    }
                    div { class: "footer-col",
                        span { class: "card-eyebrow", "Links" }
                        a { href: "https://crates.io/crates/dioxus-code", "crates.io ↗" }
                        a { href: "https://docs.rs/dioxus-code", "docs.rs ↗" }
                        a { href: "https://github.com/", "GitHub ↗" }
                    }
                    div { class: "footer-col",
                        span { class: "card-eyebrow", "Built on" }
                        a { href: "https://dioxuslabs.com", "Dioxus 0.7 ↗" }
                        a { href: "https://tree-sitter.github.io", "Tree-sitter ↗" }
                        span { class: "footer-meta", "MIT licensed" }
                    }
                }
                div { class: "footer-rule" }
                p { class: "footer-fineprint",
                    "© 2026 dioxus-code. The component, not the editor."
                }
            }
        }
    }
}

#[derive(Clone, Copy)]
struct DemoTheme {
    theme: Theme,
    accent: &'static str,
}

fn demo_themes() -> &'static [DemoTheme] {
    &[
        DemoTheme {
            theme: Theme::TOKYO_NIGHT,
            accent: "#7aa2f7",
        },
        DemoTheme {
            theme: Theme::RUSTDOC_AYU,
            accent: "#ffb454",
        },
        DemoTheme {
            theme: Theme::GITHUB_LIGHT,
            accent: "#0969da",
        },
        DemoTheme {
            theme: Theme::MELANGE_DARK,
            accent: "#e49b5d",
        },
        DemoTheme {
            theme: Theme::KANAGAWA_DRAGON,
            accent: "#c5c9c5",
        },
    ]
}

const APP_CSS: &str = r#"
@import url('https://fonts.googleapis.com/css2?family=Geist:wght@300..900&family=Geist+Mono:wght@400;500;600;700&display=swap');

:root {
  color-scheme: light;
  --bg: #fafaf6;
  --bg-tint: #f4f3ed;
  --card: #ffffff;
  --line: rgba(28, 25, 23, 0.07);
  --line-strong: rgba(28, 25, 23, 0.14);
  --ink: #1c1917;
  --ink-soft: rgba(28, 25, 23, 0.65);
  --ink-mute: rgba(28, 25, 23, 0.5);
  --accent: #6366f1;
  --paper: #fafaf6;
  --paper-soft: rgba(250, 250, 246, 0.7);
  --shadow-card: 0 1px 3px rgba(28, 25, 23, 0.04);
  --shadow-elev: 0 8px 24px -10px rgba(28, 25, 23, 0.16);
  --radius-card: 22px;
  --radius-inner: 12px;
  --max-width: 1340px;
}

html {
  scroll-behavior: smooth;
}

html,
body,
#main {
  margin: 0;
  min-height: 100%;
}

body {
  background: var(--bg);
  color: var(--ink);
  font-family: 'Geist', system-ui, -apple-system, BlinkMacSystemFont, sans-serif;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
}

* {
  box-sizing: border-box;
}

a {
  color: inherit;
  text-decoration: none;
}

button {
  cursor: pointer;
  font: inherit;
}

.site-shell {
  min-height: 100vh;
}

/* ============ Topbar ============ */

.topbar {
  align-items: center;
  background: rgba(250, 250, 246, 0.78);
  backdrop-filter: blur(18px);
  -webkit-backdrop-filter: blur(18px);
  border-bottom: 1px solid var(--line);
  display: flex;
  gap: 24px;
  justify-content: space-between;
  min-height: 60px;
  padding: 0 28px;
  position: sticky;
  top: 0;
  z-index: 30;
}

.brand,
.topbar nav {
  align-items: center;
  display: flex;
  gap: 6px;
}

.brand {
  font-family: 'Geist', sans-serif;
  font-size: 15px;
  font-weight: 600;
  gap: 12px;
  letter-spacing: -0.01em;
}

.brand-mark {
  align-items: center;
  background: #0a0a0a;
  border-radius: 8px;
  color: #fff;
  display: inline-flex;
  font-family: 'Geist Mono', monospace;
  font-size: 11px;
  font-weight: 600;
  height: 28px;
  justify-content: center;
  width: 28px;
}

.topbar nav a {
  border-radius: 8px;
  color: var(--ink-soft);
  font-family: 'Geist', sans-serif;
  font-size: 13px;
  font-weight: 500;
  padding: 8px 12px;
  transition: background 0.15s, color 0.15s;
}

.topbar nav a:hover {
  background: rgba(28, 25, 23, 0.05);
  color: var(--ink);
}

.topbar-cta {
  background: var(--ink);
  color: var(--paper) !important;
  margin-left: 6px;
}

.topbar-cta:hover {
  background: #0a0a0a !important;
  color: #fff !important;
}

/* ============ Section shell ============ */

.section {
  padding: 24px 24px;
  width: 100%;
}

.section-head {
  align-items: end;
  display: flex;
  gap: 32px;
  justify-content: space-between;
  margin: 0 auto 18px;
  max-width: var(--max-width);
  padding: 32px 6px 0;
}

.section-eyebrow {
  color: var(--accent);
  display: block;
  font-family: 'Geist Mono', monospace;
  font-size: 12px;
  font-weight: 500;
  letter-spacing: 0.04em;
  margin-bottom: 12px;
}

.section-title {
  font-family: 'Geist', sans-serif;
  font-size: clamp(28px, 3.6vw, 44px);
  font-weight: 600;
  letter-spacing: -0.03em;
  line-height: 1.05;
  margin: 0;
  max-width: 26ch;
}

.section-sub {
  color: var(--ink-soft);
  font-family: 'Geist', sans-serif;
  font-size: 15px;
  line-height: 1.55;
  margin: 0;
  max-width: 46ch;
  text-align: right;
}

/* ============ Card primitives ============ */

.card {
  background: var(--card);
  border: 1px solid var(--line);
  border-radius: var(--radius-card);
  box-shadow: var(--shadow-card);
  display: grid;
  position: relative;
}

.card-eyebrow {
  color: var(--ink-mute);
  display: block;
  font-family: 'Geist Mono', monospace;
  font-size: 11px;
  font-weight: 500;
  letter-spacing: 0.1em;
  text-transform: uppercase;
}

.card-eyebrow-light {
  color: rgba(250, 250, 246, 0.55);
}

.card-bar {
  align-items: center;
  border-bottom: 1px solid var(--line);
  color: var(--ink-mute);
  display: flex;
  font-family: 'Geist Mono', monospace;
  font-size: 11px;
  font-weight: 500;
  justify-content: space-between;
  letter-spacing: 0.08em;
  min-height: 42px;
  padding: 0 18px;
  text-transform: uppercase;
}

.card-code-body {
  overflow: auto;
}

.card-code-body .dxc {
  background: transparent;
  border: 0;
  font-family: 'Geist Mono', monospace;
  font-size: 13px;
  line-height: 1.65;
  margin: 0;
  padding: 18px 20px;
}

.card-note {
  color: var(--ink-soft);
  font-family: 'Geist', sans-serif;
  font-size: 13px;
  line-height: 1.5;
  margin: 0;
}

.cta {
  border-radius: 999px;
  display: inline-flex;
  font-family: 'Geist', sans-serif;
  font-size: 14px;
  font-weight: 500;
  padding: 11px 20px;
  transition: transform 0.15s, background 0.15s, color 0.15s, border-color 0.15s;
}

.cta.primary {
  background: var(--paper);
  color: #0a0a0a;
}

.cta.primary:hover {
  background: #fff;
  transform: translateY(-1px);
}

.cta:not(.primary) {
  background: rgba(250, 250, 246, 0.08);
  border: 1px solid rgba(250, 250, 246, 0.18);
  color: rgba(250, 250, 246, 0.92);
}

.cta:not(.primary):hover {
  background: rgba(250, 250, 246, 0.16);
}

/* ============ Hero ============ */

.hero {
  padding-top: 16px;
}

.hero-grid {
  display: grid;
  gap: 14px;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  grid-template-rows: minmax(220px, 1fr) minmax(220px, 1fr) minmax(160px, auto) minmax(160px, auto);
  margin: 0 auto;
  max-width: var(--max-width);
  width: 100%;
}

.card-pitch {
  align-content: center;
  background:
    radial-gradient(ellipse at 80% 0%, rgba(99, 102, 241, 0.18), transparent 55%),
    radial-gradient(ellipse at 0% 100%, rgba(244, 114, 182, 0.1), transparent 55%),
    linear-gradient(135deg, #0a0a0a 0%, #1c1917 100%);
  border: 0;
  color: var(--paper);
  grid-column: 1 / span 2;
  grid-row: 1 / span 2;
  padding: 44px;
}

.pitch-title {
  color: var(--paper);
  font-family: 'Geist', sans-serif;
  font-size: clamp(36px, 4.6vw, 64px);
  font-weight: 600;
  letter-spacing: -0.035em;
  line-height: 1;
  margin: 22px 0 20px;
  max-width: 14ch;
}

.pitch-lede {
  color: rgba(250, 250, 246, 0.7);
  font-family: 'Geist', sans-serif;
  font-size: 17px;
  line-height: 1.5;
  margin: 0 0 28px;
  max-width: 50ch;
}

.pitch-actions {
  display: flex;
  gap: 10px;
  margin-bottom: 32px;
}

.pitch-meta {
  align-items: center;
  color: rgba(250, 250, 246, 0.5);
  display: flex;
  font-family: 'Geist Mono', monospace;
  font-size: 12px;
  gap: 8px;
}

.pitch-meta-dot {
  background: #34d399;
  border-radius: 50%;
  box-shadow: 0 0 8px rgba(52, 211, 153, 0.7);
  height: 7px;
  width: 7px;
}

.card-code {
  grid-column: 3;
  grid-row: 1 / span 2;
  grid-template-rows: auto 1fr;
  overflow: hidden;
  padding: 0;
}

.card-install {
  grid-column: 1;
  grid-row: 3;
  padding: 24px;
  align-content: start;
}

.card-install .card-eyebrow {
  margin-bottom: 14px;
}

.shell-cmd {
  background: var(--bg-tint);
  border-radius: var(--radius-inner);
  color: var(--ink);
  display: block;
  font-family: 'Geist Mono', monospace;
  font-size: 15px;
  font-weight: 500;
  margin: 0 0 14px;
  overflow-x: auto;
  padding: 12px 14px;
}

.card-modes {
  grid-column: 2 / span 2;
  grid-row: 3;
  padding: 24px;
  align-content: start;
}

.card-modes .card-eyebrow {
  margin-bottom: 14px;
}

.modes-row {
  display: grid;
  gap: 14px;
  grid-template-columns: 1fr 1fr;
}

.mode-cell {
  background: var(--bg-tint);
  border-radius: var(--radius-inner);
  display: grid;
  gap: 6px;
  padding: 16px;
}

.mode-name {
  color: var(--ink);
  font-family: 'Geist Mono', monospace;
  font-size: 17px;
  font-weight: 600;
  margin: 0;
}

.mode-desc {
  color: var(--ink-soft);
  font-family: 'Geist', sans-serif;
  font-size: 13px;
}

.mode-tag {
  align-self: start;
  background: rgba(99, 102, 241, 0.1);
  border-radius: 999px;
  color: var(--accent);
  font-family: 'Geist Mono', monospace;
  font-size: 11px;
  font-weight: 500;
  letter-spacing: 0.02em;
  margin-top: 6px;
  padding: 4px 10px;
}

.card-zero {
  grid-column: 1;
  grid-row: 4;
  padding: 24px;
  align-content: start;
}

.card-zero .card-eyebrow {
  margin-bottom: 14px;
}

.stat-num {
  color: var(--ink);
  font-family: 'Geist', sans-serif;
  font-size: 64px;
  font-weight: 600;
  letter-spacing: -0.045em;
  line-height: 1;
  margin: 0 0 14px;
}

.card-themes {
  grid-column: 2 / span 2;
  grid-row: 4;
  padding: 24px;
  align-content: start;
}

.card-themes .card-eyebrow {
  margin-bottom: 16px;
}

.swatches {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  margin-bottom: 14px;
}

.swatch-chip {
  border-radius: 8px;
  height: 24px;
  width: 24px;
}

.swatch-more {
  align-items: center;
  background: var(--bg-tint);
  border-radius: 8px;
  color: var(--ink-soft);
  display: inline-flex;
  font-family: 'Geist Mono', monospace;
  font-size: 11px;
  font-weight: 500;
  height: 24px;
  justify-content: center;
  padding: 0 8px;
}

/* ============ Playground ============ */

.playground-grid {
  display: grid;
  gap: 14px;
  grid-template-columns: minmax(0, 1fr) minmax(0, 1.05fr);
  grid-template-rows: minmax(420px, 1fr) auto;
  margin: 0 auto;
  max-width: var(--max-width);
  width: 100%;
}

.card-editor,
.card-preview {
  grid-row: 1;
  grid-template-rows: auto minmax(0, 1fr);
  overflow: hidden;
  padding: 0;
}

.card-themepicker {
  grid-column: 1 / span 2;
  grid-row: 2;
  grid-template-rows: auto auto;
  overflow: hidden;
  padding: 0;
}

.code-input {
  background: #0c0c0c;
  border: 0;
  color: #f3eadb;
  font: 14px/1.65 'Geist Mono', ui-monospace, SFMono-Regular, Menlo, monospace;
  min-height: 420px;
  outline: none;
  padding: 20px;
  resize: vertical;
  tab-size: 4;
}

.code-input:focus {
  box-shadow: inset 0 0 0 2px var(--accent);
}

.card-preview .card-code-body {
  min-height: 420px;
}

.preview-meta {
  align-items: center;
  display: inline-flex;
  gap: 8px;
}

.preview-swatch {
  border-radius: 50%;
  height: 10px;
  width: 10px;
}

.theme-strip {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  padding: 18px 18px 20px;
}

.theme-pill {
  align-items: center;
  background: var(--bg-tint);
  border: 1px solid transparent;
  border-radius: 999px;
  color: var(--ink-soft);
  display: inline-flex;
  font-family: 'Geist', sans-serif;
  font-size: 12px;
  font-weight: 500;
  gap: 8px;
  min-height: 32px;
  padding: 0 12px 0 8px;
  transition: border-color 0.15s, color 0.15s, background 0.15s;
}

.theme-pill:hover {
  border-color: var(--line-strong);
  color: var(--ink);
}

.theme-pill.active {
  background: var(--ink);
  border-color: var(--ink);
  color: var(--paper);
}

.theme-pill-swatch {
  border-radius: 50%;
  height: 12px;
  width: 12px;
}

/* ============ Demos ============ */

.demos-grid {
  display: grid;
  gap: 14px;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  grid-template-rows: minmax(0, 1fr) minmax(0, 1fr);
  margin: 0 auto;
  max-width: var(--max-width);
  width: 100%;
}

.card-demo {
  grid-template-rows: auto minmax(0, 1fr) auto;
  overflow: hidden;
  padding: 0;
}

.card-demo-feature {
  align-content: start;
  background: var(--card);
  display: grid;
  grid-column: 1 / span 2;
  grid-row: 1 / span 2;
  grid-template-rows: auto auto auto auto;
  padding: 32px 32px 20px;
}

.demo-feature-head {
  align-items: center;
  display: flex;
  gap: 16px;
  justify-content: space-between;
  margin-bottom: 22px;
}

.demo-feature-tag {
  background: rgba(99, 102, 241, 0.1);
  border-radius: 999px;
  color: var(--accent);
  font-family: 'Geist Mono', monospace;
  font-size: 11px;
  font-weight: 500;
  letter-spacing: 0.02em;
  padding: 4px 10px;
}

.demo-feature-title {
  font-family: 'Geist', sans-serif;
  font-size: clamp(24px, 2.6vw, 36px);
  font-weight: 600;
  letter-spacing: -0.025em;
  line-height: 1.1;
  margin: 0 0 14px;
  max-width: 18ch;
}

.demo-feature-copy {
  color: var(--ink-soft);
  font-family: 'Geist', sans-serif;
  font-size: 15px;
  line-height: 1.55;
  margin: 0 0 22px;
  max-width: 60ch;
}

.demo-feature-frame {
  background: var(--bg-tint);
  border: 1px solid var(--line);
  border-radius: var(--radius-inner);
  display: grid;
  grid-template-rows: auto 1fr;
  margin: 0 -8px -8px;
  overflow: hidden;
}

.demo-feature-frame .card-bar {
  background: var(--card);
  border-bottom: 1px solid var(--line);
}

.demo-foot {
  border-top: 1px solid var(--line);
  padding: 16px 18px;
}

/* ============ Docs ============ */

.docs-grid {
  display: grid;
  gap: 14px;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  margin: 0 auto;
  max-width: var(--max-width);
  width: 100%;
}

.card-doc {
  align-content: start;
  display: grid;
  gap: 0;
  padding: 28px;
}

.doc-head {
  align-items: center;
  display: flex;
  gap: 14px;
  margin-bottom: 16px;
}

.doc-num {
  background: var(--ink);
  border-radius: 8px;
  color: var(--paper);
  font-family: 'Geist Mono', monospace;
  font-size: 12px;
  font-weight: 600;
  padding: 4px 8px;
}

.doc-title {
  font-family: 'Geist', sans-serif;
  font-size: 22px;
  font-weight: 600;
  letter-spacing: -0.02em;
  line-height: 1.15;
  margin: 0 0 12px;
}

.doc-copy {
  color: var(--ink-soft);
  font-family: 'Geist', sans-serif;
  font-size: 14px;
  line-height: 1.55;
  margin: 0 0 20px;
}

.doc-frame {
  background: var(--bg-tint);
  border-radius: var(--radius-inner);
  border: 1px solid var(--line);
  overflow: hidden;
}

.doc-frame .dxc {
  background: transparent;
  border: 0;
  font-family: 'Geist Mono', monospace;
  font-size: 12.5px;
  line-height: 1.65;
  margin: 0;
  padding: 16px 18px;
}

/* ============ Footer ============ */

.site-footer {
  padding-top: 16px;
  padding-bottom: 32px;
}

.card-footer {
  background:
    radial-gradient(ellipse at 90% 0%, rgba(99, 102, 241, 0.18), transparent 60%),
    linear-gradient(140deg, #0a0a0a 0%, #1c1917 100%);
  border: 0;
  color: var(--paper);
  margin: 0 auto;
  max-width: var(--max-width);
  padding: 40px 40px 28px;
  width: 100%;
}

.footer-grid {
  display: grid;
  gap: 32px;
  grid-template-columns: minmax(0, 1.6fr) repeat(3, minmax(0, 1fr));
}

.footer-brand-row {
  align-items: center;
  display: flex;
  gap: 12px;
}

.footer-brand-name {
  color: var(--paper);
  font-family: 'Geist', sans-serif;
  font-size: 17px;
  font-weight: 600;
  letter-spacing: -0.01em;
}

.footer-tag {
  color: rgba(250, 250, 246, 0.6);
  font-family: 'Geist', sans-serif;
  font-size: 14px;
  line-height: 1.55;
  margin: 14px 0 0;
  max-width: 32ch;
}

.footer-col {
  display: grid;
  gap: 10px;
  align-content: start;
}

.footer-col .card-eyebrow {
  color: rgba(250, 250, 246, 0.5);
  margin-bottom: 4px;
}

.footer-col a {
  color: rgba(250, 250, 246, 0.85);
  font-family: 'Geist', sans-serif;
  font-size: 14px;
  font-weight: 500;
  transition: color 0.15s;
}

.footer-col a:hover {
  color: var(--paper);
}

.footer-meta {
  color: rgba(250, 250, 246, 0.55);
  font-family: 'Geist Mono', monospace;
  font-size: 12px;
}

.footer-rule {
  border-top: 1px solid rgba(250, 250, 246, 0.12);
  margin: 32px 0 20px;
}

.footer-fineprint {
  color: rgba(250, 250, 246, 0.45);
  font-family: 'Geist Mono', monospace;
  font-size: 12px;
  margin: 0;
}

/* ============ Responsive ============ */

@media (max-width: 1100px) {
  .hero-grid {
    grid-template-columns: repeat(2, minmax(0, 1fr));
    grid-template-rows: auto;
  }

  .card-pitch {
    grid-column: 1 / span 2;
    grid-row: auto;
  }

  .card-code {
    grid-column: 1 / span 2;
    grid-row: auto;
  }

  .card-install {
    grid-column: 1;
    grid-row: auto;
  }

  .card-modes {
    grid-column: 2;
    grid-row: auto;
  }

  .card-zero {
    grid-column: 1;
    grid-row: auto;
  }

  .card-themes {
    grid-column: 2;
    grid-row: auto;
  }

  .playground-grid {
    grid-template-columns: 1fr;
    grid-template-rows: auto auto auto;
  }

  .card-editor,
  .card-preview {
    grid-row: auto;
  }

  .card-themepicker {
    grid-column: 1;
    grid-row: auto;
  }

  .demos-grid,
  .docs-grid {
    grid-template-columns: 1fr;
  }

  .card-demo-feature {
    grid-column: 1;
    grid-row: auto;
  }

  .footer-grid {
    grid-template-columns: 1fr 1fr;
  }
}

@media (max-width: 720px) {
  .topbar {
    align-items: start;
    flex-direction: column;
    gap: 8px;
    padding: 14px 18px;
  }

  .topbar nav {
    flex-wrap: wrap;
  }

  .section {
    padding: 16px 14px;
  }

  .section-head {
    align-items: start;
    flex-direction: column;
    padding: 24px 6px 0;
  }

  .section-sub {
    text-align: left;
  }

  .hero-grid {
    grid-template-columns: 1fr;
  }

  .card-pitch,
  .card-code,
  .card-install,
  .card-modes,
  .card-zero,
  .card-themes {
    grid-column: 1;
  }

  .card-pitch {
    padding: 32px 24px;
  }

  .modes-row {
    grid-template-columns: 1fr;
  }

  .footer-grid {
    grid-template-columns: 1fr;
  }

  .card-footer {
    padding: 28px 24px 22px;
  }

  .demo-feature-head {
    flex-direction: column;
    align-items: start;
    gap: 8px;
  }
}

.dxc {
  font-size: 14px;
  line-height: 1.55;
  overflow: auto;
}
"#;
