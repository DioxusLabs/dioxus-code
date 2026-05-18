use std::collections::HashSet;

use dioxus::prelude::*;
use dioxus_code::{Code, CodeTheme, SourceCode, Theme};
use dioxus_code_editor::{CodeEditor, Language};

mod components;
#[cfg(not(feature = "server"))]
mod theme;

use components::badge::{Badge, BadgeVariant};
use components::card::Card;
use components::navbar::{Navbar, NavbarContent, NavbarItem, NavbarNav, NavbarTrigger};
use components::select::{
    Select, SelectItemIndicator, SelectList, SelectOption, SelectTrigger, SelectValue,
};
use components::separator::Separator;
use components::toggle_group::{ToggleGroup, ToggleItem};

const STARTER: &str = include_str!("../snippets/starter.rs");
const HERO_COUNTER: &str = r#"use dioxus::prelude::*;

#[component]
pub fn Counter() -> Element {
    let mut count = use_signal(|| 0);

    rsx! {
        button {
            onclick: move |_| count += 1,
            "count: {count}"
        }
    }
}
"#;
const DOCS_INSTALL: &str = include_str!("../snippets/install.toml");
const DOCS_RUNTIME: &str = include_str!("../snippets/runtime.rs");
const DOCS_STATIC: &str = include_str!("../snippets/static_macro.rs");

const COMPONENTS_THEME_CSS: Asset = asset!("/assets/dx-components-theme.css");

const DEMO_THEME_PAIRS: &[ThemePair] = &[
    ThemePair::new(
        "github-light",
        Theme::GITHUB_LIGHT,
        "github-dark",
        Theme::GITHUB_DARK,
    ),
    ThemePair::new("alabaster", Theme::ALABASTER, "zenburn", Theme::ZENBURN),
    ThemePair::new("ayu-light", Theme::AYU_LIGHT, "ayu-dark", Theme::AYU_DARK),
    ThemePair::new(
        "catppuccin-latte",
        Theme::CATPPUCCIN_LATTE,
        "catppuccin-mocha",
        Theme::CATPPUCCIN_MOCHA,
    ),
    ThemePair::new("dayfox", Theme::DAYFOX, "tokyo-night", Theme::TOKYO_NIGHT),
    ThemePair::new(
        "gruvbox-light",
        Theme::GRUVBOX_LIGHT,
        "gruvbox-dark",
        Theme::GRUVBOX_DARK,
    ),
    ThemePair::new("light-owl", Theme::LIGHT_OWL, "dracula", Theme::DRACULA),
    ThemePair::new(
        "lucius-light",
        Theme::LUCIUS_LIGHT,
        "cobalt2",
        Theme::COBALT2,
    ),
    ThemePair::new(
        "melange-light",
        Theme::MELANGE_LIGHT,
        "melange-dark",
        Theme::MELANGE_DARK,
    ),
    ThemePair::new(
        "rustdoc-light",
        Theme::RUSTDOC_LIGHT,
        "rustdoc-ayu",
        Theme::RUSTDOC_AYU,
    ),
    ThemePair::new(
        "solarized-light",
        Theme::SOLARIZED_LIGHT,
        "solarized-dark",
        Theme::SOLARIZED_DARK,
    ),
];

#[derive(Clone, Copy, PartialEq, Eq)]
enum Scheme {
    System,
    Light,
    Dark,
}

impl Scheme {
    fn toggle_index(self) -> usize {
        match self {
            Scheme::System => 0,
            Scheme::Light => 1,
            Scheme::Dark => 2,
        }
    }

    fn from_toggle_index(index: usize) -> Self {
        match index {
            1 => Scheme::Light,
            2 => Scheme::Dark,
            _ => Scheme::System,
        }
    }
}

#[derive(Clone, Copy)]
struct ThemePair {
    light_name: &'static str,
    light: Theme,
    dark_name: &'static str,
    dark: Theme,
}

impl ThemePair {
    const fn new(
        light_name: &'static str,
        light: Theme,
        dark_name: &'static str,
        dark: Theme,
    ) -> Self {
        Self {
            light_name,
            light,
            dark_name,
            dark,
        }
    }

    fn code_theme(self, scheme: Scheme) -> CodeTheme {
        match scheme {
            Scheme::System => CodeTheme::system(self.light, self.dark),
            Scheme::Light => CodeTheme::fixed(self.light),
            Scheme::Dark => CodeTheme::fixed(self.dark),
        }
    }

    fn display_name(self, scheme: Scheme) -> String {
        match scheme {
            Scheme::System => self.option_name(),
            Scheme::Light => self.light_name.to_string(),
            Scheme::Dark => self.dark_name.to_string(),
        }
    }

    fn option_name(self) -> String {
        format!("{} / {}", self.light_name, self.dark_name)
    }
}

#[cfg(not(feature = "server"))]
fn main() {
    dioxus::launch(App);
}

#[cfg(feature = "server")]
fn main() {
    use dioxus::server::axum::{Json, Router, routing::post};
    use dioxus::server::{DioxusRouterExt, IncrementalRendererConfig, ServeConfig};

    dioxus::server::serve(|| async {
        let cfg = ServeConfig::builder()
            .incremental(
                IncrementalRendererConfig::new()
                    .static_dir(
                        std::env::current_exe()
                            .unwrap()
                            .parent()
                            .unwrap()
                            .join("public"),
                    )
                    .clear_cache(false),
            )
            .enable_out_of_order_streaming();

        let router = Router::new()
            .route(
                "/api/static_routes",
                post(|| async { Json(static_routes()) }),
            )
            .serve_dioxus_application(cfg, App);

        Ok(router)
    })
}

#[cfg(feature = "server")]
fn static_routes() -> Vec<String> {
    Route::static_routes()
        .iter()
        .map(ToString::to_string)
        .collect()
}

#[component]
fn App() -> Element {
    #[cfg(not(feature = "server"))]
    use_hook(theme::theme_seed);

    rsx! {
        document::Link { rel: "stylesheet", href: COMPONENTS_THEME_CSS }
        Router::<Route> {}
    }
}

/// Force the children to only render on the client
#[component]
fn ClientOnly(children: Element) -> Element {
    let mut on_client = use_signal(|| false);

    use_effect(move || on_client.set(true));

    if on_client() {
        children
    } else {
        rsx! {}
    }
}

#[derive(Routable, Clone, PartialEq)]
enum Route {
    #[route("/")]
    Home {},
}

#[component]
fn Home() -> Element {
    let source = use_signal(|| STARTER.to_string());
    let active_theme = use_signal(|| 0usize);
    let mut scheme = use_signal(|| Scheme::System);

    #[cfg(not(feature = "server"))]
    {
        use_future(move || async move {
            scheme.set(theme::read_cookie_scheme().await);
        });
    }

    let scheme_value = scheme();
    let theme_pairs = demo_theme_pairs();
    let active_theme_index = active_theme().min(theme_pairs.len() - 1);
    let active_theme_pair = theme_pairs[active_theme_index];
    let hero_theme = active_theme_pair.code_theme(scheme_value);
    let hero_theme_label = active_theme_pair.display_name(scheme_value);

    rsx! {
        document::Link { rel: "stylesheet", href: APP_CSS }
        main { class: "site-shell",
            Header { scheme }
            Hero { theme: hero_theme, theme_label: hero_theme_label }
            FeatureRowReceipt {}
            Playground { source, active_theme, scheme: scheme_value }
            Docs { scheme: scheme_value }
            SiteFooter {}
        }
    }
}

#[component]
fn Header(scheme: Signal<Scheme>) -> Element {
    rsx! {
        header { class: "topbar",
            a { class: "brand", href: "#top", "aria-label": "Homepage",
                span { class: "brand-mark",
                    IconDioxus {}
                }
                span { "dioxus-code" }
            }
            Navbar { "aria-label": "Main navigation",
                NavbarItem {
                    index: 0usize,
                    value: "features".to_string(),
                    to: "#features",
                    "Features"
                }
                NavbarItem {
                    index: 1usize,
                    value: "playground".to_string(),
                    to: "#playground",
                    "Playground"
                }
                NavbarItem {
                    index: 2usize,
                    value: "docs".to_string(),
                    to: "#docs",
                    "Docs"
                }
                NavbarNav { index: 3usize,
                    NavbarTrigger { "Resources" }
                    NavbarContent {
                        NavbarItem {
                            index: 0usize,
                            value: "crates".to_string(),
                            to: "https://crates.io/crates/dioxus-code",
                            new_tab: true,
                            "crates.io"
                            IconExternal {}
                        }
                        NavbarItem {
                            index: 1usize,
                            value: "docs".to_string(),
                            to: "https://docs.rs/dioxus-code",
                            new_tab: true,
                            "docs.rs"
                            IconExternal {}
                        }
                        NavbarItem {
                            index: 2usize,
                            value: "github".to_string(),
                            to: "https://github.com/ealmloff/dioxus-code",
                            new_tab: true,
                            "GitHub"
                            IconExternal {}
                        }
                    }
                }
            }
            div { class: "topbar-tail",
                ThemeToggle { scheme }
            }
        }
    }
}

#[component]
fn ThemeToggle(mut scheme: Signal<Scheme>) -> Element {
    let pressed_set = use_memo(move || Some(HashSet::from([scheme().toggle_index()])));

    rsx! {
        ToggleGroup {
            "aria-label": "Color scheme",
            horizontal: true,
            pressed: pressed_set,
            on_pressed_change: move |set: HashSet<usize>| {
                if let Some(&idx) = set.iter().next() {
                    let new = Scheme::from_toggle_index(idx);
                    scheme.set(new);
                    #[cfg(not(feature = "server"))]
                    theme::set_scheme(new);
                }
            },
            ToggleItem {
                index: 0usize,
                "aria-label": "Use system color scheme",
                title: "System",
                IconMonitor {}
            }
            ToggleItem {
                index: 1usize,
                "aria-label": "Light color scheme",
                title: "Light",
                IconSun {}
            }
            ToggleItem {
                index: 2usize,
                "aria-label": "Dark color scheme",
                title: "Dark",
                IconMoon {}
            }
        }
    }
}

#[component]
fn CopyCommandButton(command: &'static str) -> Element {
    let mut copied = use_signal(|| false);

    let on_click = move |_| {
        let escaped = command.replace('\\', "\\\\").replace('"', "\\\"");
        let script = format!(
            r#"
            try {{ await navigator.clipboard.writeText("{escaped}"); }} catch (e) {{}}
            dioxus.send(true);
            await new Promise(r => setTimeout(r, 1600));
            dioxus.send(false);
            "#
        );
        spawn(async move {
            let mut eval = document::eval(&script);
            while let Ok(state) = eval.recv::<bool>().await {
                copied.set(state);
            }
        });
    };

    rsx! {
        button {
            r#type: "button",
            class: "term-copy",
            "data-copied": copied().then_some("true"),
            "aria-label": if copied() { "Copied" } else { "Copy command" },
            title: if copied() { "Copied" } else { "Copy command" },
            onclick: on_click,
            if copied() {
                IconCheck {}
                span { class: "term-copy-label", "Copied" }
            } else {
                IconCopy {}
                span { class: "term-copy-label", "Copy" }
            }
        }
    }
}

#[component]
fn IconCopy() -> Element {
    rsx! {
        svg {
            width: "14",
            height: "14",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "1.7",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            "aria-hidden": "true",
            rect { x: "9", y: "9", width: "12", height: "12", rx: "2.5" }
            path { d: "M5 15H4a1 1 0 0 1-1-1V4a1 1 0 0 1 1-1h10a1 1 0 0 1 1 1v1" }
        }
    }
}

#[component]
fn IconCheck() -> Element {
    rsx! {
        svg {
            width: "14",
            height: "14",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            "aria-hidden": "true",
            path { d: "M5 12.5l4.5 4.5L19 7" }
        }
    }
}

#[component]
fn IconSun() -> Element {
    rsx! {
        svg {
            width: "16",
            height: "16",
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
            width: "16",
            height: "16",
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

const DIOXUS_ICON: Asset = asset!("/assets/dioxus_color.svg");

#[component]
fn IconDioxus() -> Element {
    rsx! {
        img { src: DIOXUS_ICON, alt: "Dioxus" }
    }
}

#[component]
fn IconMonitor() -> Element {
    rsx! {
        svg {
            width: "16",
            height: "16",
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
fn IconExternal() -> Element {
    rsx! {
        svg {
            class: "external-icon",
            width: "12",
            height: "12",
            view_box: "0 0 24 24",
            fill: "currentColor",
            stroke: "currentColor",
            stroke_width: "0.25",
            "aria-hidden": "true",
            path { d: "M5 21q-.825 0-1.412-.587T3 19V5q0-.825.588-1.412T5 3h7v2H5v14h14v-7h2v7q0 .825-.587 1.413T19 21zm4.7-5.3l-1.4-1.4L17.6 5H14V3h7v7h-2V6.4z" }
        }
    }
}

#[component]
fn Hero(theme: CodeTheme, theme_label: String) -> Element {
    rsx! {
        section { id: "top", class: "hero hero-terminal",
            div { class: "hero-terminal-grid",
                div { class: "hero-terminal-copy",
                    div { class: "hero-eyebrow",
                        Badge { variant: BadgeVariant::Outline, "v0.1 · Dioxus 0.7" }
                    }
                    h1 { class: "hero-h1",
                        "Code highlighter for Dioxus; runtime or "
                        em { "compile time" }
                        "."
                    }
                    p { class: "hero-lede",
                        "A drop-in component with two source modes: compile-time macro and runtime highlighting with explicit language selection."
                    }
                    div { class: "hero-terminal-block",
                        div { class: "hero-terminal-bar",
                            span { class: "term-dot r" }
                            span { class: "term-dot y" }
                            span { class: "term-dot g" }
                            span { class: "hero-terminal-title", "~/my-app" }
                            CopyCommandButton { command: "cargo add dioxus-code" }
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
                }
                div { class: "hero-stage hero-stage-split",
                    div { class: "card-bar",
                        span { "src/counter.rs" }
                        span { "{theme_label}" }
                    }
                    div { class: "card-code-body",
                        Code { src: SourceCode::builder(HERO_COUNTER).with_language(Language::Rust), theme }
                    }
                }
            }
            div { class: "hero-actions",
                a { class: "hero-cta hero-cta-primary", href: "#docs",
                    span { "Read the docs" }
                    svg {
                        class: "hero-cta-arrow",
                        width: "14",
                        height: "14",
                        view_box: "0 0 24 24",
                        fill: "none",
                        stroke: "currentColor",
                        stroke_width: "2",
                        stroke_linecap: "round",
                        stroke_linejoin: "round",
                        "aria-hidden": "true",
                        path { d: "M5 12h14M13 5l7 7-7 7" }
                    }
                }
                a { class: "hero-cta hero-cta-ghost", href: "#playground",
                    span { "See it live" }
                    span { class: "hero-cta-meta", "playground" }
                }
            }
        }
    }
}

#[component]
fn FeatureRowReceipt() -> Element {
    rsx! {
        section { id: "features", class: "section",
            div { class: "section-head receipt-head",
                div {
                    h2 { class: "section-title receipt-title", "What's in the crate." }
                }
            }
            div { class: "receipt-stack",
                article { class: "receipt",
                    ul { class: "receipt-items",
                        li { class: "receipt-item",
                            span { class: "receipt-label", "code! macro" }
                            span { class: "receipt-dots" }
                            span { class: "receipt-value", "INCLUDED" }
                        }
                        li { class: "receipt-item",
                            span { class: "receipt-label", "Pre-styled spans" }
                            span { class: "receipt-dots" }
                            span { class: "receipt-value", "INCLUDED" }
                        }
                        li { class: "receipt-item",
                            span { class: "receipt-label", "Themes (scoped CSS)" }
                            span { class: "receipt-dots" }
                            span { class: "receipt-value", "× 30+" }
                        }
                        li { class: "receipt-item receipt-optional",
                            span { class: "receipt-label", "SourceCode runtime" }
                            span { class: "receipt-dots" }
                            span { class: "receipt-value", "OPT-IN" }
                        }
                        li { class: "receipt-item receipt-optional",
                            span { class: "receipt-label", "Tree-sitter grammars" }
                            span { class: "receipt-dots" }
                            span { class: "receipt-value", "+3.33 MiB" }
                        }
                    }
                    div { class: "receipt-rule double" }
                    div { class: "receipt-total",
                        span { class: "receipt-total-label", "PARSER BYTES SHIPPED" }
                        span { class: "receipt-total-value", "0" }
                    }
                }
                aside { class: "receipt-aside",
                    div { class: "receipt-aside-row",
                        span { class: "receipt-aside-num", "01" }
                        div {
                            h3 { class: "receipt-aside-title", "code!" }
                            p { class: "receipt-aside-text", "Tokenizes during cargo build. The runtime gets pre-styled markup with no parser bytes." }
                        }
                    }
                    div { class: "receipt-aside-row",
                        span { class: "receipt-aside-num", "02" }
                        div {
                            h3 { class: "receipt-aside-title", "SourceCode" }
                            p { class: "receipt-aside-text", "Pull it in when input is dynamic. Pass the language you want to highlight." }
                        }
                    }
                    div { class: "receipt-aside-row",
                        span { class: "receipt-aside-num", "03" }
                        div {
                            h3 { class: "receipt-aside-title", "Themes" }
                            p { class: "receipt-aside-text", "Tokyo Night, Catppuccin, Dracula, Rosé Pine, GitHub… each one is scoped CSS, mix several on a page." }
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
    scheme: Scheme,
) -> Element {
    let theme_pairs = demo_theme_pairs();
    let active_idx = use_memo(move || active_theme().min(theme_pairs.len() - 1));
    let theme_pair = theme_pairs[active_idx()];
    let theme = theme_pair.code_theme(scheme);
    let value = use_memo(move || Some(active_idx()));
    let language = Language::detect(&source()).unwrap_or(Language::Rust);
    let language_label = language.slug();
    let source_len = source().chars().count();

    rsx! {
        section { id: "playground", class: "section",
            div { class: "section-head",
                div {
                    h2 { class: "section-title", "Edit highlighted code inline." }
                }
            }
            div { class: "playground-grid",
                Card { class: "card-editor",
                    div { class: "card-bar",
                        span { "source" }
                        span { class: "editor-meta",
                            span { "{language_label} · {source_len} chars" }
                            span { class: "editor-meta-divider" }
                            Select::<usize> {
                                value: Some(value.into()),
                                on_value_change: move |v: Option<usize>| {
                                    if let Some(idx) = v {
                                        active_theme.set(idx);
                                    }
                                },
                                SelectTrigger {
                                    SelectValue { placeholder: "Choose a theme" }
                                }
                                SelectList {
                                    for (i, pair) in theme_pairs.iter().copied().enumerate() {
                                        SelectOption::<usize> {
                                            value: i,
                                            text_value: pair.option_name(),
                                            index: i,
                                            span { "{pair.option_name()}" }
                                            SelectItemIndicator {}
                                        }
                                    }
                                }
                            }
                        }
                    }
                    ClientOnly {
                        CodeEditor {
                            value: source(),
                            language,
                            theme,
                            aria_label: "Source editor",
                            placeholder: "Type code...",
                            class: "playground-code-editor",
                            oninput: move |value| source.set(value),
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn Docs(scheme: Scheme) -> Element {
    let theme_pair = ThemePair::new(
        "github-light",
        Theme::GITHUB_LIGHT,
        "github-dark",
        Theme::GITHUB_DARK,
    );
    let theme = theme_pair.code_theme(scheme);
    let theme_label = theme_pair.display_name(scheme);
    let steps = doc_step_data();

    rsx! {
        section { id: "docs", class: "section",
            div { class: "section-head",
                div {
                    h2 { class: "section-title", "Get started" }
                }
            }
            ol { class: "docs-timeline",
                for step in steps.iter() {
                    li { class: "docs-timeline-step",
                        div { class: "docs-timeline-rail",
                            span { class: "docs-timeline-num", "{step.num}" }
                        }
                        div { class: "docs-timeline-content",
                            span { class: "card-eyebrow", "{step.eyebrow}" }
                            h3 { class: "docs-timeline-title", "{step.title}" }
                            p { class: "docs-timeline-copy", "{step.copy}" }
                            div { class: "docs-timeline-frame",
                                div { class: "card-bar",
                                    span { "{step.file_name}" }
                                    span { "{theme_label}" }
                                }
                                div { class: "card-code-body",
                                    Code {
                                        src: SourceCode::builder(step.code).with_language(step.language),
                                        theme,
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[derive(Clone, Copy)]
struct DocStepData {
    num: &'static str,
    eyebrow: &'static str,
    title: &'static str,
    copy: &'static str,
    code: &'static str,
    language: Language,
    file_name: &'static str,
}

fn doc_step_data() -> [DocStepData; 3] {
    [
        DocStepData {
            num: "01",
            eyebrow: "Install",
            title: "Add the dependency",
            copy: "Enable the runtime feature when source comes from user input, generated files, or network responses.",
            code: DOCS_INSTALL,
            language: Language::Toml,
            file_name: "Cargo.toml",
        },
        DocStepData {
            num: "02",
            eyebrow: "Runtime source",
            title: "SourceCode for live input",
            copy: "Pass any string through SourceCode. Provide a language hint when you already know it — Arborium handles tokenizing.",
            code: DOCS_RUNTIME,
            language: Language::Rust,
            file_name: "runtime.rs",
        },
        DocStepData {
            num: "03",
            eyebrow: "Static source",
            title: "code! for snippets in your repo",
            copy: "Use the macro for examples, docs, and any source checked in alongside your app. Highlight markup is generated at compile time.",
            code: DOCS_STATIC,
            language: Language::Rust,
            file_name: "static.rs",
        },
    ]
}

#[component]
fn SiteFooter() -> Element {
    rsx! {
        footer { class: "section site-footer",
            div { class: "card-footer",
                div { class: "footer-grid",
                    div { class: "footer-brand",
                        div { class: "footer-brand-row",
                            span { class: "brand-mark",
                                IconDioxus {}
                            }
                            span { class: "footer-brand-name", "dioxus-code" }
                        }
                        p { class: "footer-tag",
                            "Syntax highlighting, designed for the inside of your Dioxus app."
                        }
                    }
                    div { class: "footer-col",
                        span { class: "card-eyebrow", "Project" }
                        a { href: "#features", "Features" }
                        a { href: "#playground", "Playground" }
                        a { href: "#docs", "Documentation" }
                    }
                    div { class: "footer-col",
                        span { class: "card-eyebrow", "Links" }
                        a { href: "https://crates.io/crates/dioxus-code",
                            "crates.io"
                            IconExternal {}
                        }
                        a { href: "https://docs.rs/dioxus-code",
                            "docs.rs"
                            IconExternal {}
                        }
                        a { href: "https://github.com/ealmloff/dioxus-code",
                            "GitHub"
                            IconExternal {}
                        }
                    }
                    div { class: "footer-col",
                        span { class: "card-eyebrow", "Built on" }
                        a { href: "https://tree-sitter.github.io",
                            "Tree-sitter"
                            IconExternal {}
                        }
                        a { href: "https://crates.io/crates/arborium",
                            "Arborium"
                            IconExternal {}
                        }
                        span { class: "footer-meta", "MIT licensed" }
                    }
                }
                Separator { class: "footer-separator", horizontal: true }
                p { class: "footer-fineprint", "© 2026 dioxus-code" }
            }
        }
    }
}

fn demo_theme_pairs() -> &'static [ThemePair] {
    DEMO_THEME_PAIRS
}

const APP_CSS: Asset = asset!("/assets/app.css");
