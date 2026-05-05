use dioxus::prelude::*;
use dioxus_code::{Code, SourceCode, Theme, code};
use dioxus_code_editor::CodeEditor;

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

const DOCS_RUNTIME: &str = r#"use dioxus_code::{Code, SourceCode, Theme};

rsx! {
    Code {
        src: SourceCode::new(source).with_language("rust"),
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

const SYSTEM_THEME_SCRIPT: &str = r#"
const media = window.matchMedia("(prefers-color-scheme: dark)");
const send = () => dioxus.send(media.matches);
send();

if (media.addEventListener) {
    media.addEventListener("change", send);
} else {
    media.addListener(send);
}

await new Promise(() => {});
"#;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Scheme {
    System,
    Light,
    Dark,
}

impl Scheme {
    fn class(self) -> &'static str {
        match self {
            Scheme::System => "site-shell theme-system",
            Scheme::Light => "site-shell theme-light",
            Scheme::Dark => "site-shell theme-dark",
        }
    }

    fn resolved(self, system_theme: ThemeMode) -> ThemeMode {
        match self {
            Scheme::System => system_theme,
            Scheme::Light => ThemeMode::Light,
            Scheme::Dark => ThemeMode::Dark,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ThemeMode {
    Light,
    Dark,
}

impl ThemeMode {
    fn from_is_dark(is_dark: bool) -> Self {
        if is_dark {
            ThemeMode::Dark
        } else {
            ThemeMode::Light
        }
    }

    fn demo_themes(self) -> &'static [DemoTheme] {
        match self {
            ThemeMode::Light => light_demo_themes(),
            ThemeMode::Dark => dark_demo_themes(),
        }
    }

    fn pick(self, light: Theme, dark: Theme) -> Theme {
        match self {
            ThemeMode::Light => light,
            ThemeMode::Dark => dark,
        }
    }
}

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    let source = use_signal(|| STARTER.to_string());
    let active_theme = use_signal(|| 0usize);
    let scheme = use_signal(|| Scheme::System);
    let mut system_theme = use_signal(|| ThemeMode::Light);

    use_future(move || async move {
        let mut eval = document::eval(SYSTEM_THEME_SCRIPT);

        while let Ok(is_dark) = eval.recv::<bool>().await {
            system_theme.set(ThemeMode::from_is_dark(is_dark));
        }
    });

    let theme_mode = scheme().resolved(system_theme());
    let themes = theme_mode.demo_themes();
    let active_theme_index = active_theme().min(themes.len() - 1);

    rsx! {
        style { {APP_CSS} }
        main { class: scheme().class(),
            Header { scheme }
            Hero { source: source(), theme: themes[active_theme_index].theme }
            SizeCharts {}
            Playground { source, active_theme, theme_mode }
            Demos { theme_mode }
            Docs { theme_mode }
            SiteFooter {}
        }
    }
}

#[component]
fn Header(scheme: Signal<Scheme>) -> Element {
    rsx! {
        header { class: "topbar",
            a { class: "brand", href: "#top", "aria-label": "Homepage",
                span { class: "brand-mark", "dx" }
                span { "dioxus-code" }
            }
            nav {
                a { href: "#sizes", "Size" }
                a { href: "#playground", "Playground" }
                a { href: "#demos", "Demos" }
                a { href: "#docs", "Docs" }
                ThemeToggle { scheme }
                a { class: "topbar-cta", href: "https://crates.io/crates/dioxus-code", "crates.io ↗" }
            }
        }
    }
}

#[component]
fn ThemeToggle(mut scheme: Signal<Scheme>) -> Element {
    rsx! {
        div { class: "theme-toggle", role: "group", "aria-label": "Color scheme",
            button {
                class: if scheme() == Scheme::System { "theme-seg active" } else { "theme-seg" },
                title: "System",
                "aria-label": "Use system color scheme",
                onclick: move |_| scheme.set(Scheme::System),
                IconMonitor {}
            }
            button {
                class: if scheme() == Scheme::Light { "theme-seg active" } else { "theme-seg" },
                title: "Light",
                "aria-label": "Light color scheme",
                onclick: move |_| scheme.set(Scheme::Light),
                IconSun {}
            }
            button {
                class: if scheme() == Scheme::Dark { "theme-seg active" } else { "theme-seg" },
                title: "Dark",
                "aria-label": "Dark color scheme",
                onclick: move |_| scheme.set(Scheme::Dark),
                IconMoon {}
            }
        }
    }
}

#[component]
fn IconSun() -> Element {
    rsx! {
        svg {
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "1.7",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            "aria-hidden": "true",
            circle { cx: "12", cy: "12", r: "4" }
            path { d: "M12 3v1.5M12 19.5V21M4.22 4.22l1.06 1.06M18.72 18.72l1.06 1.06M3 12h1.5M19.5 12H21M4.22 19.78l1.06-1.06M18.72 5.28l1.06-1.06" }
        }
    }
}

#[component]
fn IconMoon() -> Element {
    rsx! {
        svg {
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "1.7",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            "aria-hidden": "true",
            path { d: "M21 12.6A9 9 0 1 1 11.4 3a7 7 0 0 0 9.6 9.6z" }
        }
    }
}

#[component]
fn IconMonitor() -> Element {
    rsx! {
        svg {
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "1.7",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            "aria-hidden": "true",
            rect { x: "3", y: "4.5", width: "18", height: "12", rx: "2" }
            path { d: "M8.5 20h7M12 16.5V20" }
        }
    }
}

#[component]
fn Hero(source: String, theme: Theme) -> Element {
    rsx! {
        section { id: "top", class: "hero hero-terminal",
            div { class: "hero-terminal-grid",
                div { class: "hero-terminal-copy",
                    h1 { class: "hero-h1",
                        "Highlight code in Dioxus, with one "
                        em { "cargo add" }
                        "."
                    }
                    p { class: "hero-lede",
                        "A drop-in component with two source modes — compile-time macro and runtime detection. No JS, no flash of unstyled code."
                    }
                    div { class: "hero-terminal-block",
                        div { class: "hero-terminal-bar",
                            span { class: "term-dot r" }
                            span { class: "term-dot y" }
                            span { class: "term-dot g" }
                            span { class: "hero-terminal-title", "~/my-app" }
                        }
                        div { class: "hero-terminal-body",
                            p { class: "term-line",
                                span { class: "term-prompt", "$" }
                                span { "cargo add dioxus-code" }
                            }
                            p { class: "term-line term-output",
                                "    Updating crates.io index"
                            }
                            p { class: "term-line term-output",
                                "    Adding dioxus-code v0.1 to dependencies"
                            }
                            p { class: "term-line term-success",
                                "    Done."
                            }
                        }
                    }
                    div { class: "hero-actions",
                        a { class: "hero-cta primary", href: "#docs", "Read the docs →" }
                        a { class: "hero-cta", href: "#playground", "See it live" }
                    }
                }
                div { class: "hero-stage hero-stage-split",
                    div { class: "card-bar",
                        span { "src/counter.rs" }
                        span { "{theme.name()}" }
                    }
                    div { class: "card-code-body",
                        Code { src: SourceCode::new(source).with_language("rust"), theme }
                    }
                }
            }
        }
    }
}

#[derive(Clone, Copy)]
struct BuildDelta {
    example: &'static str,
    mode: &'static str,
    delta: &'static str,
    detail: &'static str,
    width: &'static str,
    accent: &'static str,
}

fn build_deltas() -> &'static [BuildDelta] {
    &[
        BuildDelta {
            example: "Macro only",
            mode: "compile-time",
            delta: "+0.20 MiB",
            detail: "12% over baseline",
            width: "6.09%",
            accent: "#16a34a",
        },
        BuildDelta {
            example: "Basic",
            mode: "runtime",
            delta: "+3.33 MiB",
            detail: "198% over baseline",
            width: "100%",
            accent: "#dc2626",
        },
    ]
}

#[component]
fn SizeCharts() -> Element {
    rsx! {
        section { id: "sizes", class: "section",
            div { class: "section-head",
                div {
                    h2 { class: "section-title", "opt into runtime parsing" }
                }
                p { class: "section-sub",
                    "Release web WASM over the Dioxus hello baseline."
                }
            }
            div { class: "size-panel",
                div { class: "size-source",
                    span { "dx build --web -r" }
                    span { "WASM over baseline" }
                }
                div { class: "chart-block",
                    div { class: "chart-head",
                        span { class: "card-eyebrow", "Over baseline" }
                        span { class: "chart-scale", "max +3.33 MiB" }
                    }
                    div { class: "size-bars",
                        for build in build_deltas() {
                            div { class: "size-row",
                                div { class: "size-row-label",
                                    strong { "{build.example}" }
                                    span { "{build.mode}" }
                                }
                                div { class: "size-track", role: "img", "aria-label": "{build.example} adds {build.delta} over baseline",
                                    div {
                                        class: "size-bar",
                                        style: "width:{build.width}; background:{build.accent};",
                                    }
                                }
                                div { class: "size-row-value",
                                    strong { "{build.delta}" }
                                    span { "{build.detail}" }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn Playground(
    mut source: Signal<String>,
    mut active_theme: Signal<usize>,
    theme_mode: ThemeMode,
) -> Element {
    let themes = theme_mode.demo_themes();
    let active_idx = active_theme().min(themes.len() - 1);
    let theme = themes[active_idx].theme;
    let active_swatch = themes[active_idx].accent;
    rsx! {
        section { id: "playground", class: "section",
            div { class: "section-head",
                div {
                    h2 { class: "section-title", "Edit highlighted code inline." }
                }
                p { class: "section-sub",
                    "Type Rust in the contenteditable editor, swap themes, and keep the rendered output in one surface."
                }
            }
            div { class: "playground-grid",
                div { class: "card card-editor",
                    div { class: "card-bar",
                        span { "source.rs" }
                        span { class: "editor-meta",
                            span { "rust · " {format!("{} chars", source().chars().count())} }
                            span { class: "editor-meta-divider" }
                            span { class: "editor-swatch", style: "background:{active_swatch};" }
                            "{theme.name()}"
                        }
                    }
                    CodeEditor {
                        value: source(),
                        language: "rust",
                        name: "source.rs",
                        theme,
                        aria_label: "Rust source editor",
                        placeholder: "Type Rust code...",
                        class: "playground-code-editor",
                        oninput: move |value| source.set(value),
                    }
                }
                div { class: "card card-themepicker",
                    div { class: "card-bar",
                        span { "active theme" }
                        span { {format!("{} of {}", active_idx + 1, themes.len())} }
                    }
                    div { class: "theme-strip",
                        for (index, swatch) in themes.iter().enumerate() {
                            button {
                                class: if active_idx == index { "theme-pill active" } else { "theme-pill" },
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
fn Demos(theme_mode: ThemeMode) -> Element {
    let feature_theme = theme_mode.pick(Theme::MELANGE_LIGHT, Theme::KANAGAWA_DRAGON);
    let runtime_theme = theme_mode.pick(Theme::GITHUB_LIGHT, Theme::GITHUB_DARK);

    rsx! {
        section { id: "demos", class: "section",
            div { class: "section-head",
                div {
                    h2 { class: "section-title", "Examples" }
                }
                p { class: "section-sub",
                    "Static snippets at compile time, runtime parsing when the source is dynamic."
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
                            span { "{feature_theme.name()}" }
                        }
                        div { class: "card-code-body",
                            Code {
                                src: code!("/snippets/palette.rs"),
                                theme: feature_theme,
                            }
                        }
                    }
                }
                article { class: "card card-demo card-demo-runtime",
                    div { class: "card-bar",
                        span { "runtime · python" }
                        span { "{runtime_theme.name()}" }
                    }
                    div { class: "card-code-body",
                        Code {
                            src: SourceCode::new(PYTHON).with_language("python"),
                            theme: runtime_theme,
                        }
                    }
                    div { class: "demo-foot",
                        p { class: "card-note",
                            "Pass any string with a known language. Tree-sitter grammars cover Rust, JS, Python, Go, and dozens more."
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn Docs(theme_mode: ThemeMode) -> Element {
    let install_theme = theme_mode.pick(Theme::MELANGE_LIGHT, Theme::MELANGE_DARK);
    let runtime_theme = theme_mode.pick(Theme::RUSTDOC_LIGHT, Theme::RUSTDOC_AYU);
    let static_theme = theme_mode.pick(Theme::GITHUB_LIGHT, Theme::TOKYO_NIGHT);

    rsx! {
        section { id: "docs", class: "section",
            div { class: "section-head",
                div {
                    h2 { class: "section-title", "Get started" }
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
                    theme: install_theme,
                }
                DocStep {
                    id: "runtime",
                    num: "02",
                    eyebrow: "Runtime source",
                    title: "SourceCode for live input",
                    copy: "Pass any string through SourceCode. Provide a language hint when you already know it — Arborium handles tokenizing.",
                    code: DOCS_RUNTIME,
                    language: "rust",
                    theme: runtime_theme,
                }
                DocStep {
                    id: "static",
                    num: "03",
                    eyebrow: "Static source",
                    title: "code! for snippets in your repo",
                    copy: "Use the macro for examples, docs, and any source checked in alongside your app. Highlight markup is generated at compile time.",
                    code: DOCS_STATIC,
                    language: "rust",
                    theme: static_theme,
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
                    src: SourceCode::new(code).with_language(language),
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
                        a { href: "#sizes", "Release size" }
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

fn light_demo_themes() -> &'static [DemoTheme] {
    &[
        DemoTheme {
            theme: Theme::ALABASTER,
            accent: "#aa3731",
        },
        DemoTheme {
            theme: Theme::AYU_LIGHT,
            accent: "#f07171",
        },
        DemoTheme {
            theme: Theme::CATPPUCCIN_LATTE,
            accent: "#1e66f5",
        },
        DemoTheme {
            theme: Theme::DAYFOX,
            accent: "#2848a9",
        },
        DemoTheme {
            theme: Theme::GITHUB_LIGHT,
            accent: "#0969da",
        },
        DemoTheme {
            theme: Theme::GRUVBOX_LIGHT,
            accent: "#076678",
        },
        DemoTheme {
            theme: Theme::LIGHT_OWL,
            accent: "#403f53",
        },
        DemoTheme {
            theme: Theme::LUCIUS_LIGHT,
            accent: "#005f87",
        },
        DemoTheme {
            theme: Theme::MELANGE_LIGHT,
            accent: "#c47f2c",
        },
        DemoTheme {
            theme: Theme::RUSTDOC_LIGHT,
            accent: "#7c3aed",
        },
        DemoTheme {
            theme: Theme::SOLARIZED_LIGHT,
            accent: "#268bd2",
        },
    ]
}

fn dark_demo_themes() -> &'static [DemoTheme] {
    &[
        DemoTheme {
            theme: Theme::AYU_DARK,
            accent: "#ff8f40",
        },
        DemoTheme {
            theme: Theme::CATPPUCCIN_FRAPPE,
            accent: "#8caaee",
        },
        DemoTheme {
            theme: Theme::CATPPUCCIN_MACCHIATO,
            accent: "#8aadf4",
        },
        DemoTheme {
            theme: Theme::CATPPUCCIN_MOCHA,
            accent: "#89b4fa",
        },
        DemoTheme {
            theme: Theme::COBALT2,
            accent: "#ffc600",
        },
        DemoTheme {
            theme: Theme::DESERT256,
            accent: "#ffd700",
        },
        DemoTheme {
            theme: Theme::DRACULA,
            accent: "#bd93f9",
        },
        DemoTheme {
            theme: Theme::EF_MELISSA_DARK,
            accent: "#ef9fe4",
        },
        DemoTheme {
            theme: Theme::GITHUB_DARK,
            accent: "#58a6ff",
        },
        DemoTheme {
            theme: Theme::GRUVBOX_DARK,
            accent: "#83a598",
        },
        DemoTheme {
            theme: Theme::KANAGAWA_DRAGON,
            accent: "#c5c9c5",
        },
        DemoTheme {
            theme: Theme::MELANGE_DARK,
            accent: "#e49b5d",
        },
        DemoTheme {
            theme: Theme::MONOKAI,
            accent: "#66d9ef",
        },
        DemoTheme {
            theme: Theme::NORD,
            accent: "#88c0d0",
        },
        DemoTheme {
            theme: Theme::ONE_DARK,
            accent: "#61afef",
        },
        DemoTheme {
            theme: Theme::ROSE_PINE_MOON,
            accent: "#c4a7e7",
        },
        DemoTheme {
            theme: Theme::RUSTDOC_AYU,
            accent: "#ffb454",
        },
        DemoTheme {
            theme: Theme::RUSTDOC_DARK,
            accent: "#2bab63",
        },
        DemoTheme {
            theme: Theme::SOLARIZED_DARK,
            accent: "#268bd2",
        },
        DemoTheme {
            theme: Theme::TOKYO_NIGHT,
            accent: "#7aa2f7",
        },
        DemoTheme {
            theme: Theme::ZENBURN,
            accent: "#efef8f",
        },
    ]
}

const APP_CSS: &str = r#"
@import url('https://fonts.googleapis.com/css2?family=Geist:wght@300..900&family=Geist+Mono:wght@400;500;600;700&display=swap');

:root,
.theme-light {
  --bg: #fafaf6;
  --bg-tint: #f4f3ed;
  --card: #ffffff;
  --line: rgba(28, 25, 23, 0.07);
  --line-strong: rgba(28, 25, 23, 0.14);
  --ink: #1c1917;
  --ink-soft: rgba(28, 25, 23, 0.65);
  --ink-mute: rgba(28, 25, 23, 0.5);
  --accent: #6366f1;
  --accent-soft: rgba(99, 102, 241, 0.1);
  --surface-soft: rgba(28, 25, 23, 0.06);
  --topbar-bg: rgba(250, 250, 246, 0.78);
  --feature-bg: linear-gradient(135deg, #0a0a0a 0%, #1c1917 100%);
  --feature-mesh-1: radial-gradient(ellipse at 80% 0%, rgba(99, 102, 241, 0.18), transparent 55%);
  --feature-mesh-2: radial-gradient(ellipse at 0% 100%, rgba(244, 114, 182, 0.1), transparent 55%);
  --feature-bg-footer: linear-gradient(140deg, #0a0a0a 0%, #1c1917 100%);
  --feature-mesh-footer: radial-gradient(ellipse at 90% 0%, rgba(99, 102, 241, 0.18), transparent 60%);
  --feature-text: #fafaf6;
  --feature-soft: rgba(250, 250, 246, 0.7);
  --feature-mute: rgba(250, 250, 246, 0.5);
  --feature-line: rgba(250, 250, 246, 0.12);
  --feature-cta-bg: #fafaf6;
  --feature-cta-fg: #0a0a0a;
  --feature-cta-ghost-bg: rgba(250, 250, 246, 0.08);
  --feature-cta-ghost-line: rgba(250, 250, 246, 0.18);
  --feature-cta-ghost-fg: rgba(250, 250, 246, 0.92);
  --code-bg: #ffffff;
  --editor-bg: #ffffff;
  --editor-fg: #1f2328;
  --editor-gutter-bg: #f6f8fa;
  --editor-gutter-fg: rgba(31, 35, 40, 0.42);
  --editor-gutter-line: rgba(31, 35, 40, 0.1);
  --editor-selection: rgba(9, 105, 218, 0.2);
  --shadow-card: 0 1px 3px rgba(28, 25, 23, 0.04);
  --shadow-elev: 0 8px 24px -10px rgba(28, 25, 23, 0.16);
  --radius-card: 22px;
  --radius-inner: 12px;
  --max-width: 1340px;
  color-scheme: light;
}

:root:has(.theme-dark),
html:has(.theme-dark) {
  --bg: #0c0a08;
  --bg-tint: #1a1612;
  --card: #1a1612;
  --line: rgba(255, 255, 255, 0.07);
  --line-strong: rgba(255, 255, 255, 0.16);
  --ink: #f5f3ee;
  --ink-soft: rgba(245, 243, 238, 0.7);
  --ink-mute: rgba(245, 243, 238, 0.5);
  --accent: #a5b4fc;
  --accent-soft: rgba(165, 180, 252, 0.14);
  --surface-soft: rgba(255, 255, 255, 0.06);
  --topbar-bg: rgba(12, 10, 8, 0.78);
  --feature-bg: #1a1612;
  --feature-mesh-1: none;
  --feature-mesh-2: none;
  --feature-bg-footer: #1a1612;
  --feature-mesh-footer: none;
  --feature-text: #f5f3ee;
  --feature-soft: rgba(245, 243, 238, 0.72);
  --feature-mute: rgba(245, 243, 238, 0.48);
  --feature-line: rgba(255, 255, 255, 0.08);
  --feature-cta-bg: #f5f3ee;
  --feature-cta-fg: #0a0a0a;
  --feature-cta-ghost-bg: rgba(245, 243, 238, 0.06);
  --feature-cta-ghost-line: rgba(245, 243, 238, 0.16);
  --feature-cta-ghost-fg: rgba(245, 243, 238, 0.92);
  --code-bg: #0d1117;
  --editor-bg: #0d1117;
  --editor-fg: #e6edf3;
  --editor-gutter-bg: rgba(255, 255, 255, 0.03);
  --editor-gutter-fg: rgba(230, 237, 243, 0.42);
  --editor-gutter-line: rgba(255, 255, 255, 0.08);
  --editor-selection: rgba(122, 162, 247, 0.34);
  --shadow-card: none;
  --shadow-elev: none;
  color-scheme: dark;
}

@media (prefers-color-scheme: dark) {
  :root:has(.theme-system),
  html:has(.theme-system) {
    --bg: #0c0a08;
    --bg-tint: #1a1612;
    --card: #1a1612;
    --line: rgba(255, 255, 255, 0.07);
    --line-strong: rgba(255, 255, 255, 0.16);
    --ink: #f5f3ee;
    --ink-soft: rgba(245, 243, 238, 0.7);
    --ink-mute: rgba(245, 243, 238, 0.5);
    --accent: #a5b4fc;
    --accent-soft: rgba(165, 180, 252, 0.14);
    --surface-soft: rgba(255, 255, 255, 0.06);
    --topbar-bg: rgba(12, 10, 8, 0.78);
    --feature-bg: #1a1612;
    --feature-mesh-1: none;
    --feature-mesh-2: none;
    --feature-bg-footer: #1a1612;
    --feature-mesh-footer: none;
    --feature-text: #f5f3ee;
    --feature-soft: rgba(245, 243, 238, 0.72);
    --feature-mute: rgba(245, 243, 238, 0.48);
    --feature-line: rgba(255, 255, 255, 0.08);
    --feature-cta-bg: #f5f3ee;
    --feature-cta-fg: #0a0a0a;
    --feature-cta-ghost-bg: rgba(245, 243, 238, 0.06);
    --feature-cta-ghost-line: rgba(245, 243, 238, 0.16);
    --feature-cta-ghost-fg: rgba(245, 243, 238, 0.92);
    --code-bg: #0d1117;
    --editor-bg: #0d1117;
    --editor-fg: #e6edf3;
    --editor-gutter-bg: rgba(255, 255, 255, 0.03);
    --editor-gutter-fg: rgba(230, 237, 243, 0.42);
    --editor-gutter-line: rgba(255, 255, 255, 0.08);
    --editor-selection: rgba(122, 162, 247, 0.34);
    --shadow-card: none;
    --shadow-elev: none;
    color-scheme: dark;
  }
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
  background: var(--topbar-bg);
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
  background: var(--ink);
  border-radius: 8px;
  color: var(--bg);
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
  background: var(--accent-soft);
  color: var(--ink);
}

.topbar-cta {
  background: var(--ink) !important;
  color: var(--bg) !important;
  margin-left: 6px;
}

.topbar-cta:hover {
  filter: brightness(1.08);
}

/* Theme toggle */

.theme-toggle {
  align-items: center;
  background: var(--surface-soft);
  border: 1px solid var(--line);
  border-radius: 999px;
  display: inline-flex;
  gap: 2px;
  margin: 0 4px 0 8px;
  padding: 3px;
}

.theme-seg {
  align-items: center;
  background: transparent;
  border: 0;
  border-radius: 999px;
  color: var(--ink-mute);
  cursor: pointer;
  display: inline-flex;
  height: 26px;
  justify-content: center;
  padding: 0;
  transition: background 0.15s, color 0.15s;
  width: 28px;
}

.theme-seg:hover {
  color: var(--ink);
}

.theme-seg.active {
  background: var(--card);
  box-shadow: 0 1px 2px rgba(0, 0, 0, 0.08);
  color: var(--ink);
}

.theme-dark .theme-seg.active,
.theme-system .theme-seg.active {
  box-shadow: none;
}

.theme-seg svg {
  height: 14px;
  width: 14px;
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
  color: var(--feature-mute);
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
  background: var(--code-bg);
  overflow: auto;
}

.card-code-body .dxc {
  background: var(--code-bg);
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
  background: var(--feature-cta-bg);
  color: var(--feature-cta-fg);
}

.cta.primary:hover {
  filter: brightness(1.04);
  transform: translateY(-1px);
}

.cta:not(.primary) {
  background: var(--feature-cta-ghost-bg);
  border: 1px solid var(--feature-cta-ghost-line);
  color: var(--feature-cta-ghost-fg);
}

.cta:not(.primary):hover {
  background: var(--feature-cta-ghost-line);
}

/* ============ Hero (shared primitives) ============ */

.hero {
  margin: 0 auto;
  max-width: var(--max-width);
  padding: 32px 24px 56px;
  width: 100%;
}

.hero-h1 {
  color: var(--ink);
  font-family: 'Geist', sans-serif;
  font-size: clamp(40px, 5.6vw, 80px);
  font-weight: 600;
  letter-spacing: -0.04em;
  line-height: 0.98;
  margin: 0 0 22px;
  max-width: 18ch;
  text-wrap: balance;
}

.hero-h1 em {
  color: var(--accent);
  font-family: 'Geist Mono', monospace;
  font-style: normal;
  font-weight: 500;
  letter-spacing: -0.02em;
}

.hero-lede {
  color: var(--ink-soft);
  font-family: 'Geist', sans-serif;
  font-size: 18px;
  line-height: 1.55;
  margin: 0 0 28px;
  max-width: 56ch;
  text-wrap: pretty;
}

.hero-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 10px;
}

.hero-cta {
  border: 1px solid transparent;
  border-radius: 999px;
  display: inline-flex;
  font-family: 'Geist', sans-serif;
  font-size: 14px;
  font-weight: 500;
  padding: 11px 22px;
  transition: transform 0.15s, background 0.15s, border-color 0.15s, color 0.15s, filter 0.15s;
}

.hero-cta.primary {
  background: var(--ink);
  color: var(--bg);
}

.hero-cta.primary:hover {
  filter: brightness(1.1);
  transform: translateY(-1px);
}

.hero-cta:not(.primary) {
  border-color: var(--line-strong);
  color: var(--ink);
}

.hero-cta:not(.primary):hover {
  background: var(--surface-soft);
}

.hero-stage {
  background: var(--card);
  border: 1px solid var(--line);
  border-radius: var(--radius-card);
  box-shadow: var(--shadow-card);
  display: grid;
  grid-template-rows: auto minmax(0, 1fr);
  overflow: hidden;
}

.hero-stage .card-code-body .dxc {
  background: var(--code-bg);
  border: 0;
  font-family: 'Geist Mono', monospace;
  font-size: 13px;
  line-height: 1.65;
  margin: 0;
  min-height: 380px;
  padding: 18px 20px;
}

.hero-stage-split {
  align-self: stretch;
}

.hero-stage-split .card-code-body .dxc {
  min-height: 460px;
}

/* Hero: Terminal install */

.hero-terminal-grid {
  align-items: center;
  display: grid;
  gap: 56px;
  grid-template-columns: minmax(0, 1fr) minmax(0, 1.1fr);
  padding: 32px 0;
}

.hero-terminal-copy {
  align-content: center;
  display: grid;
}

.hero-terminal-block {
  background: #0c0c0c;
  border: 1px solid var(--line);
  border-radius: var(--radius-card);
  margin: 4px 0 26px;
  max-width: 540px;
  overflow: hidden;
}

.hero-terminal-bar {
  align-items: center;
  background: rgba(255, 255, 255, 0.04);
  border-bottom: 1px solid rgba(255, 255, 255, 0.08);
  display: flex;
  gap: 8px;
  min-height: 36px;
  padding: 0 14px;
}

.term-dot {
  border-radius: 50%;
  height: 11px;
  width: 11px;
}

.term-dot.r { background: #ff5f57; }
.term-dot.y { background: #febc2e; }
.term-dot.g { background: #28c840; }

.hero-terminal-title {
  color: rgba(255, 255, 255, 0.5);
  font-family: 'Geist Mono', monospace;
  font-size: 12px;
  margin-left: 8px;
}

.hero-terminal-body {
  color: #f3eadb;
  font-family: 'Geist Mono', monospace;
  font-size: 13px;
  line-height: 1.85;
  padding: 18px 20px;
}

.term-line {
  margin: 0;
  white-space: pre;
}

.term-prompt {
  color: #a5b4fc;
  font-weight: 600;
  margin-right: 10px;
}

.term-output {
  color: rgba(243, 234, 219, 0.55);
}

.term-success {
  color: #34d399;
}

/* ============ Size charts ============ */

.size-panel {
  background: var(--card);
  border: 1px solid var(--line);
  border-radius: var(--radius-card);
  box-shadow: var(--shadow-card);
  display: grid;
  gap: 22px;
  margin: 0 auto;
  max-width: var(--max-width);
  overflow: hidden;
  padding: 24px;
  width: 100%;
}

.size-source {
  align-items: center;
  border-bottom: 1px solid var(--line);
  color: var(--ink-mute);
  display: flex;
  flex-wrap: wrap;
  font-family: 'Geist Mono', monospace;
  font-size: 11px;
  gap: 10px;
  letter-spacing: 0.06em;
  margin: -4px -4px 0;
  padding: 0 4px 18px;
  text-transform: uppercase;
}

.size-source span {
  background: var(--bg-tint);
  border-radius: 8px;
  padding: 6px 8px;
}

.chart-block {
  display: grid;
  gap: 16px;
  min-width: 0;
}

.chart-head {
  align-items: center;
  display: flex;
  gap: 14px;
  justify-content: space-between;
}

.chart-scale {
  color: var(--ink-mute);
  font-family: 'Geist Mono', monospace;
  font-size: 11px;
  letter-spacing: 0.06em;
  text-transform: uppercase;
}

.size-bars {
  display: grid;
  gap: 14px;
}

.size-row {
  align-items: center;
  display: grid;
  gap: 14px;
  grid-template-columns: minmax(150px, 0.7fr) minmax(0, 1.8fr) minmax(96px, 0.45fr);
}

.size-row-label,
.size-row-value {
  display: grid;
  gap: 3px;
}

.size-row-label strong,
.size-row-value strong {
  color: var(--ink);
  font-family: 'Geist', sans-serif;
  font-size: 14px;
  font-weight: 600;
  letter-spacing: 0;
}

.size-row-label span,
.size-row-value span {
  color: var(--ink-mute);
  font-family: 'Geist Mono', monospace;
  font-size: 11px;
  letter-spacing: 0.06em;
  text-transform: uppercase;
}

.size-row-value {
  text-align: right;
}

.size-track {
  background: var(--bg-tint);
  border: 1px solid var(--line);
  border-radius: 999px;
  height: 18px;
  overflow: hidden;
  position: relative;
}

.size-track::after {
  background: linear-gradient(90deg, transparent 0, transparent calc(25% - 1px), var(--line) 25%, transparent calc(25% + 1px), transparent calc(50% - 1px), var(--line) 50%, transparent calc(50% + 1px), transparent calc(75% - 1px), var(--line) 75%, transparent calc(75% + 1px));
  content: "";
  inset: 0;
  opacity: 0.7;
  pointer-events: none;
  position: absolute;
}

.size-bar {
  border-radius: inherit;
  height: 100%;
}

/* ============ Playground ============ */

.playground-grid {
  display: grid;
  gap: 14px;
  grid-template-columns: minmax(0, 1fr);
  grid-template-rows: minmax(480px, 1fr) auto;
  margin: 0 auto;
  max-width: var(--max-width);
  width: 100%;
}

.card-editor {
  grid-row: 1;
  grid-template-rows: auto minmax(0, 1fr);
  overflow: hidden;
  padding: 0;
}

.card-themepicker {
  grid-column: 1;
  grid-row: 2;
  grid-template-rows: auto auto;
  overflow: hidden;
  padding: 0;
}

.playground-code-editor {
  --dxc-editor-caret: var(--editor-fg);
  --dxc-editor-focus-ring: inset 0 0 0 2px var(--accent);
  --dxc-editor-gutter-line-padding: 0 14px 0 18px;
  --dxc-editor-gutter-padding: 20px 0;
  --dxc-editor-gutter-width: 4ch;
  --dxc-editor-padding: 20px 22px 20px 0;
  --dxc-editor-selection: var(--editor-selection);
  background: var(--editor-bg);
  color: var(--editor-fg);
  font: 14px/1.65 'Geist Mono', ui-monospace, SFMono-Regular, Menlo, monospace;
  min-height: 480px;
  width: 100%;
}

.playground-code-editor .dxc-editor-gutter {
  background: var(--editor-gutter-bg);
  border-right: 1px solid var(--editor-gutter-line);
  color: var(--editor-gutter-fg);
}

.playground-code-editor .dxc-editor-highlight,
.playground-code-editor .dxc-editor-input {
  overflow-x: auto;
}

.editor-meta {
  align-items: center;
  display: inline-flex;
  gap: 8px;
}

.editor-meta-divider {
  background: var(--line-strong);
  display: inline-flex;
  height: 12px;
  width: 1px;
}

.editor-swatch {
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
  color: var(--bg);
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

.card-demo-runtime {
  grid-column: 3;
  grid-row: 1 / span 2;
}

.demo-feature-head {
  align-items: center;
  display: flex;
  gap: 16px;
  justify-content: space-between;
  margin-bottom: 22px;
}

.demo-feature-tag {
  background: var(--accent-soft);
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
  background: var(--code-bg);
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
  color: var(--bg);
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
  background: var(--code-bg);
  border-radius: var(--radius-inner);
  border: 1px solid var(--line);
  overflow: hidden;
}

.doc-frame .dxc {
  background: var(--code-bg);
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
    var(--feature-mesh-footer),
    var(--feature-bg-footer);
  border: 1px solid var(--feature-line);
  box-shadow: var(--shadow-card);
  color: var(--feature-text);
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

.card-footer .brand-mark {
  background: var(--feature-text);
  color: #1c1917;
}

.footer-brand-name {
  color: var(--feature-text);
  font-family: 'Geist', sans-serif;
  font-size: 17px;
  font-weight: 600;
  letter-spacing: -0.01em;
}

.footer-tag {
  color: var(--feature-soft);
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
  color: var(--feature-mute);
  margin-bottom: 4px;
}

.footer-col a {
  color: var(--feature-soft);
  font-family: 'Geist', sans-serif;
  font-size: 14px;
  font-weight: 500;
  transition: color 0.15s;
}

.footer-col a:hover {
  color: var(--feature-text);
}

.footer-meta {
  color: var(--feature-mute);
  font-family: 'Geist Mono', monospace;
  font-size: 12px;
}

.footer-rule {
  border-top: 1px solid var(--feature-line);
  margin: 32px 0 20px;
}

.footer-fineprint {
  color: var(--feature-mute);
  font-family: 'Geist Mono', monospace;
  font-size: 12px;
  margin: 0;
}

/* ============ Responsive ============ */

@media (max-width: 1100px) {
  .hero-terminal-grid {
    gap: 36px;
    grid-template-columns: 1fr;
  }

  .playground-grid {
    grid-template-columns: 1fr;
    grid-template-rows: auto auto;
  }

  .card-editor {
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

  .card-demo-runtime {
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

  .hero {
    padding: 24px 14px 40px;
  }

  .size-panel {
    padding: 18px;
  }

  .size-row {
    align-items: stretch;
    grid-template-columns: 1fr;
    gap: 8px;
  }

  .size-row-value {
    text-align: left;
  }

  .chart-head {
    align-items: start;
    flex-direction: column;
    gap: 5px;
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
