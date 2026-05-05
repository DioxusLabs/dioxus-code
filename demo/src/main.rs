use std::collections::HashSet;

use dioxus::prelude::*;
use dioxus_code::{Code, SourceCode, Theme, code};
use dioxus_code_editor::CodeEditor;
use dioxus_primitives::ContentSide;

mod components;
#[cfg(not(feature = "server"))]
mod theme;

use components::badge::{Badge, BadgeVariant};
use components::card::{Card, CardContent, CardDescription, CardHeader, CardTitle};
use components::navbar::{Navbar, NavbarContent, NavbarItem, NavbarNav, NavbarTrigger};
use components::select::{
    Select, SelectItemIndicator, SelectList, SelectOption, SelectTrigger, SelectValue,
};
use components::separator::Separator;
use components::tabs::{TabContent, TabList, TabTrigger, Tabs, TabsVariant};
use components::toggle_group::{ToggleGroup, ToggleItem};
use components::tooltip::{Tooltip, TooltipContent, TooltipTrigger};

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

#[cfg(not(feature = "server"))]
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

const COMPONENTS_THEME_CSS: Asset = asset!("/assets/dx-components-theme.css");

#[derive(Clone, Copy, PartialEq, Eq)]
enum Scheme {
    System,
    Light,
    Dark,
}

impl Scheme {
    fn resolved(self, system_theme: ThemeMode) -> ThemeMode {
        match self {
            Scheme::System => system_theme,
            Scheme::Light => ThemeMode::Light,
            Scheme::Dark => ThemeMode::Dark,
        }
    }

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

#[derive(Clone, Copy, PartialEq, Eq)]
enum ThemeMode {
    Light,
    Dark,
}

impl ThemeMode {
    #[cfg(not(feature = "server"))]
    fn from_is_dark(is_dark: bool) -> Self {
        if is_dark {
            ThemeMode::Dark
        } else {
            ThemeMode::Light
        }
    }

    fn demo_themes(self) -> &'static [Theme] {
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
    let system_theme = use_signal(|| ThemeMode::Light);

    #[cfg(not(feature = "server"))]
    {
        let mut system_theme_for_script = system_theme;
        use_future(move || async move {
            let mut eval = document::eval(SYSTEM_THEME_SCRIPT);

            while let Ok(is_dark) = eval.recv::<bool>().await {
                system_theme_for_script.set(ThemeMode::from_is_dark(is_dark));
            }
        });

        use_future(move || async move {
            scheme.set(theme::read_cookie_scheme().await);
        });
    }

    let theme_mode = scheme().resolved(system_theme());

    let themes = theme_mode.demo_themes();
    let active_theme_index = active_theme().min(themes.len() - 1);

    rsx! {
        style { {APP_CSS} }
        main { class: "site-shell",
            Header { scheme }
            Hero { source: source(), theme: themes[active_theme_index] }
            FeatureRow {}
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
            Navbar { "aria-label": "Main navigation",
                NavbarItem {
                    index: 0usize,
                    value: "features".to_string(),
                    to: "#features",
                    "Why"
                }
                NavbarItem {
                    index: 1usize,
                    value: "sizes".to_string(),
                    to: "#sizes",
                    "Size"
                }
                NavbarItem {
                    index: 2usize,
                    value: "playground".to_string(),
                    to: "#playground",
                    "Playground"
                }
                NavbarItem {
                    index: 3usize,
                    value: "demos".to_string(),
                    to: "#demos",
                    "Demos"
                }
                NavbarItem {
                    index: 4usize,
                    value: "docs".to_string(),
                    to: "#docs",
                    "Docs"
                }
                NavbarNav { index: 5usize,
                    NavbarTrigger { "Resources" }
                    NavbarContent {
                        NavbarItem {
                            index: 0usize,
                            value: "crates".to_string(),
                            to: "https://crates.io/crates/dioxus-code",
                            new_tab: true,
                            "crates.io ↗"
                        }
                        NavbarItem {
                            index: 1usize,
                            value: "docs".to_string(),
                            to: "https://docs.rs/dioxus-code",
                            new_tab: true,
                            "docs.rs ↗"
                        }
                        NavbarItem {
                            index: 2usize,
                            value: "github".to_string(),
                            to: "https://github.com/ealmloff/dioxus-code",
                            new_tab: true,
                            "GitHub ↗"
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
fn Hero(source: String, theme: Theme) -> Element {
    rsx! {
        section { id: "top", class: "hero hero-terminal",
            div { class: "hero-terminal-grid",
                div { class: "hero-terminal-copy",
                    div { class: "hero-eyebrow",
                        Badge { variant: BadgeVariant::Outline, "v0.1 · Dioxus 0.7" }
                    }
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
struct Feature {
    eyebrow: &'static str,
    title: &'static str,
    body: &'static str,
}

fn features() -> &'static [Feature] {
    &[
        Feature {
            eyebrow: "Compile-time",
            title: "Zero parser shipped",
            body: "The code! macro tokenizes during cargo build, so the runtime gets pre-styled markup with no parser bytes.",
        },
        Feature {
            eyebrow: "Runtime",
            title: "Live source, opt-in",
            body: "Pull SourceCode in when input is dynamic. Tree-sitter grammars detect language automatically.",
        },
        Feature {
            eyebrow: "Themes",
            title: "Thirty-plus, scoped",
            body: "Tokyo Night, Catppuccin, Dracula, Rosé Pine, GitHub… each theme is scoped CSS so you can mix several on a page.",
        },
    ]
}

#[component]
fn FeatureRow() -> Element {
    rsx! {
        section { id: "features", class: "section",
            div { class: "section-head",
                div {
                    h2 { class: "section-title", "Two ways to highlight." }
                }
                p { class: "section-sub",
                    "Built around dx components — every interactive surface on this page is from the dx-component registry."
                }
            }
            div { class: "feature-grid",
                for feature in features() {
                    Card { class: "feature-card",
                        CardHeader {
                            Badge { variant: BadgeVariant::Secondary, "{feature.eyebrow}" }
                            CardTitle { "{feature.title}" }
                            CardDescription { "{feature.body}" }
                        }
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
    explainer: &'static str,
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
            explainer: "code! emits pre-styled spans during cargo build. The runtime ships zero parsing logic.",
        },
        BuildDelta {
            example: "Basic",
            mode: "runtime",
            delta: "+3.33 MiB",
            detail: "198% over baseline",
            width: "100%",
            accent: "#dc2626",
            explainer: "SourceCode pulls in tree-sitter grammars. Use only when the source isn't known at build time.",
        },
    ]
}

#[component]
fn SizeCharts() -> Element {
    rsx! {
        section { id: "sizes", class: "section",
            div { class: "section-head",
                div {
                    h2 { class: "section-title", "Opt into runtime parsing" }
                }
                p { class: "section-sub",
                    "Release web WASM over the Dioxus hello baseline. Hover any row for context."
                }
            }
            Card { class: "size-card",
                CardContent { class: "size-card-content",
                    div { class: "size-source",
                        Badge { variant: BadgeVariant::Outline, "dx build --web -r" }
                        Badge { variant: BadgeVariant::Outline, "WASM over baseline" }
                    }
                    div { class: "chart-block",
                        div { class: "chart-head",
                            span { class: "card-eyebrow", "Over baseline" }
                            span { class: "chart-scale", "max +3.33 MiB" }
                        }
                        div { class: "size-bars",
                            for build in build_deltas() {
                                Tooltip {
                                    TooltipTrigger {
                                        div { class: "size-row",
                                            div { class: "size-row-label",
                                                strong { "{build.example}" }
                                                span { "{build.mode}" }
                                            }
                                            div { class: "size-track", role: "img",
                                                "aria-label": "{build.example} adds {build.delta} over baseline",
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
                                    TooltipContent { side: ContentSide::Top, "{build.explainer}" }
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
    let theme = themes[active_idx];
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
                Card { class: "card-editor",
                    div { class: "card-bar",
                        span { "source.rs" }
                        span { class: "editor-meta",
                            span { "rust · " {format!("{} chars", source().chars().count())} }
                            span { class: "editor-meta-divider" }
                            Select::<usize> {
                                value: Some(use_memo(move || Some(active_idx)).into()),
                                on_value_change: move |v: Option<usize>| {
                                    if let Some(idx) = v {
                                        active_theme.set(idx);
                                    }
                                },
                                SelectTrigger {
                                    SelectValue { placeholder: "Choose a theme" }
                                }
                                SelectList {
                                    for (i, swatch) in themes.iter().enumerate() {
                                        SelectOption::<usize> {
                                            value: i,
                                            text_value: swatch.name().to_string(),
                                            index: i,
                                            span { "{swatch.name()}" }
                                            SelectItemIndicator {}
                                        }
                                    }
                                }
                            }
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
            }
        }
    }
}

#[component]
fn Demos(theme_mode: ThemeMode) -> Element {
    let feature_theme = theme_mode.pick(Theme::MELANGE_LIGHT, Theme::KANAGAWA_DRAGON);
    let runtime_theme = theme_mode.pick(Theme::GITHUB_LIGHT, Theme::GITHUB_DARK);

    let mut active_demo = use_signal(|| "macro".to_string());
    let value = use_memo(move || Some(active_demo()));

    rsx! {
        section { id: "demos", class: "section",
            div { class: "section-head",
                div {
                    h2 { class: "section-title", "Examples" }
                }
                p { class: "section-sub",
                    "Static snippets at compile time, runtime parsing when the source is dynamic. Switch between the two."
                }
            }
            Card { class: "demos-card",
                CardContent { class: "demos-card-content",
                    Tabs {
                        variant: TabsVariant::Ghost,
                        horizontal: true,
                        value,
                        default_value: "macro".to_string(),
                        on_value_change: move |v: String| active_demo.set(v),
                        TabList {
                            TabTrigger {
                                value: "macro".to_string(),
                                index: 0usize,
                                span { class: "demo-tab-label",
                                    "Compile time"
                                    Badge { variant: BadgeVariant::Secondary, "0kb runtime" }
                                }
                            }
                            TabTrigger {
                                value: "runtime".to_string(),
                                index: 1usize,
                                span { class: "demo-tab-label",
                                    "Runtime"
                                    Badge { variant: BadgeVariant::Outline, "live source" }
                                }
                            }
                        }
                        TabContent {
                            value: "macro".to_string(),
                            index: 0usize,
                            div { class: "demo-pane",
                                div { class: "demo-pane-copy",
                                    h3 { class: "demo-pane-title", "code! tokenizes at build." }
                                    p { class: "demo-pane-body",
                                        "Point the macro at a file in your repo. Highlighting happens during cargo build, so the output is pre-styled markup. No runtime parser ships to users."
                                    }
                                }
                                div { class: "demo-pane-frame",
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
                        }
                        TabContent {
                            value: "runtime".to_string(),
                            index: 1usize,
                            div { class: "demo-pane",
                                div { class: "demo-pane-copy",
                                    h3 { class: "demo-pane-title", "SourceCode handles live input." }
                                    p { class: "demo-pane-body",
                                        "Pass any string with a language hint (or let auto-detection do it). Tree-sitter grammars cover Rust, JS, Python, Go, and dozens more."
                                    }
                                }
                                div { class: "demo-pane-frame",
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
        Card { id, class: "doc-card",
            CardHeader {
                div { class: "doc-head",
                    Badge { variant: BadgeVariant::Primary, "{num}" }
                    span { class: "card-eyebrow", "{eyebrow}" }
                }
                CardTitle { "{title}" }
                CardDescription { "{copy}" }
            }
            CardContent {
                div { class: "doc-frame",
                    Code {
                        src: SourceCode::new(code).with_language(language),
                        theme,
                    }
                }
            }
        }
    }
}

#[component]
fn SiteFooter() -> Element {
    rsx! {
        footer { class: "section site-footer",
            div { class: "card-footer",
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
                Separator { class: "footer-separator", horizontal: true }
                p { class: "footer-fineprint",
                    "© 2026 dioxus-code. The component, not the editor."
                }
            }
        }
    }
}

fn light_demo_themes() -> &'static [Theme] {
    &[
        Theme::ALABASTER,
        Theme::AYU_LIGHT,
        Theme::CATPPUCCIN_LATTE,
        Theme::DAYFOX,
        Theme::GITHUB_LIGHT,
        Theme::GRUVBOX_LIGHT,
        Theme::LIGHT_OWL,
        Theme::LUCIUS_LIGHT,
        Theme::MELANGE_LIGHT,
        Theme::RUSTDOC_LIGHT,
        Theme::SOLARIZED_LIGHT,
    ]
}

fn dark_demo_themes() -> &'static [Theme] {
    &[
        Theme::AYU_DARK,
        Theme::CATPPUCCIN_FRAPPE,
        Theme::CATPPUCCIN_MACCHIATO,
        Theme::CATPPUCCIN_MOCHA,
        Theme::COBALT2,
        Theme::DESERT256,
        Theme::DRACULA,
        Theme::EF_MELISSA_DARK,
        Theme::GITHUB_DARK,
        Theme::GRUVBOX_DARK,
        Theme::KANAGAWA_DRAGON,
        Theme::MELANGE_DARK,
        Theme::MONOKAI,
        Theme::NORD,
        Theme::ONE_DARK,
        Theme::ROSE_PINE_MOON,
        Theme::RUSTDOC_AYU,
        Theme::RUSTDOC_DARK,
        Theme::SOLARIZED_DARK,
        Theme::TOKYO_NIGHT,
        Theme::ZENBURN,
    ]
}

const APP_CSS: &str = r#"
:root {
  --bg: var(--primary-color);
  --bg-tint: var(--light, var(--primary-color-3)) var(--dark, var(--primary-color-1));
  --card: var(--light, var(--primary-color-2)) var(--dark, var(--primary-color-3));
  --line: var(--primary-color-6);
  --line-strong: var(--primary-color-7);
  --ink: var(--secondary-color-1);
  --ink-soft: var(--secondary-color-4);
  --ink-mute: var(--secondary-color-5);
  --accent: var(--focused-border-color);
  --accent-soft: rgb(43 127 255 / 14%);
  --surface-soft: var(--light, var(--primary-color-3)) var(--dark, var(--primary-color-4));
  --topbar-bg: var(--light, var(--primary-color-2)) var(--dark, var(--primary-color-3));
  --feature-bg-footer: var(--light, var(--primary-color-3)) var(--dark, var(--primary-color-4));
  --feature-text: var(--secondary-color-1);
  --feature-soft: var(--secondary-color-4);
  --feature-mute: var(--secondary-color-5);
  --feature-line: var(--primary-color-6);
  --code-bg: var(--light, var(--primary-color-2)) var(--dark, var(--primary-color-3));
  --editor-bg: var(--light, var(--primary-color-2)) var(--dark, var(--primary-color-3));
  --editor-fg: var(--secondary-color-4);
  --editor-gutter-bg: var(--light, var(--primary-color-3)) var(--dark, var(--primary-color-4));
  --editor-gutter-fg: var(--secondary-color-5);
  --editor-gutter-line: var(--primary-color-6);
  --editor-selection: rgb(43 127 255 / 24%);
  --shadow-card: var(--light, 0 1px 3px rgb(0 0 0 / 6%)) var(--dark, none);
  --shadow-elev: var(--light, 0 8px 24px -10px rgb(0 0 0 / 16%)) var(--dark, none);
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

.brand {
  align-items: center;
  display: flex;
  font-family: Inter, sans-serif;
  font-size: 15px;
  font-weight: 600;
  gap: 12px;
  letter-spacing: -0.01em;
}

.topbar-tail {
  align-items: center;
  display: flex;
  gap: 14px;
}

.brand-mark {
  align-items: center;
  background: var(--ink);
  border-radius: 8px;
  color: var(--bg);
  display: inline-flex;
  font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
  font-size: 11px;
  font-weight: 600;
  height: 28px;
  justify-content: center;
  width: 28px;
}

/* Theme toggle (uses ToggleGroup component) */


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
  color: var(--ink);
  font-family: Inter, sans-serif;
  font-size: clamp(28px, 3.6vw, 44px);
  font-weight: 600;
  letter-spacing: -0.03em;
  line-height: 1.05;
  margin: 0;
  max-width: 26ch;
}

.section-sub {
  color: var(--ink-soft);
  font-family: Inter, sans-serif;
  font-size: 15px;
  line-height: 1.55;
  margin: 0;
  max-width: 46ch;
  text-align: right;
}

/* ============ Card primitive overrides ============ */

.card-eyebrow {
  color: var(--ink-mute);
  display: block;
  font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
  font-size: 11px;
  font-weight: 500;
  letter-spacing: 0.1em;
  text-transform: uppercase;
}

.card-bar {
  align-items: center;
  border-bottom: 1px solid var(--line);
  color: var(--ink-mute);
  display: flex;
  font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
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
  font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
  font-size: 13px;
  line-height: 1.65;
  margin: 0;
  padding: 18px 20px;
}

/* ============ Hero ============ */

.hero {
  margin: 0 auto;
  max-width: var(--max-width);
  padding: 32px 24px 56px;
  width: 100%;
}

.hero-eyebrow {
  margin-bottom: 18px;
}

.hero-h1 {
  color: var(--ink);
  font-family: Inter, sans-serif;
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
  font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
  font-style: normal;
  font-weight: 500;
  letter-spacing: -0.02em;
}

.hero-lede {
  color: var(--ink-soft);
  font-family: Inter, sans-serif;
  font-size: 18px;
  line-height: 1.55;
  margin: 0 0 28px;
  max-width: 56ch;
  text-wrap: pretty;
}

.hero-actions {
  align-items: center;
  display: flex;
  flex-wrap: wrap;
  gap: 12px;
  margin-top: 4px;
}

.hero-cta {
  align-items: center;
  border-radius: 12px;
  display: inline-flex;
  font-family: Inter, sans-serif;
  font-size: 14px;
  font-weight: 600;
  gap: 8px;
  height: 42px;
  letter-spacing: -0.005em;
  padding: 0 16px 0 18px;
  transition: background 0.18s ease, border-color 0.18s ease, color 0.18s ease, transform 0.18s ease, box-shadow 0.18s ease;
  white-space: nowrap;
}

.hero-cta:focus-visible {
  outline: 2px solid var(--accent);
  outline-offset: 3px;
}

.hero-cta-primary {
  background: var(--ink);
  box-shadow: var(--light, 0 6px 18px -10px rgb(0 0 0 / 35%)) var(--dark, 0 1px 0 0 rgb(255 255 255 / 6%) inset);
  color: var(--bg);
}

.hero-cta-primary:hover {
  background: var(--accent);
  color: #fff;
}

.hero-cta-primary:hover .hero-cta-arrow {
  transform: translateX(3px);
}

.hero-cta-arrow {
  transition: transform 0.18s ease;
}

.hero-cta-ghost {
  background: transparent;
  border: 1px solid var(--line-strong);
  color: var(--ink);
  padding: 0 6px 0 16px;
}

.hero-cta-ghost:hover {
  background: var(--bg-tint);
  border-color: var(--ink);
}

.hero-cta-meta {
  background: var(--bg-tint);
  border-radius: 8px;
  color: var(--ink-mute);
  font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
  font-size: 10px;
  font-weight: 500;
  letter-spacing: 0.08em;
  padding: 4px 8px;
  text-transform: uppercase;
  transition: background 0.18s ease, color 0.18s ease;
}

.hero-cta-ghost:hover .hero-cta-meta {
  background: var(--card);
  color: var(--ink-soft);
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
  font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
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
  background: var(--light, #0c0c0c) var(--dark, #161b22);
  border: 1px solid var(--light, transparent) var(--dark, var(--primary-color-6));
  border-radius: var(--radius-card);
  margin: 4px 0 26px;
  max-width: 540px;
  overflow: hidden;
}

.hero-terminal-bar {
  align-items: center;
  background: rgb(255 255 255 / 4%);
  border-bottom: 1px solid rgb(255 255 255 / 8%);
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
  font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
  font-size: 12px;
  margin-left: 8px;
}

.hero-terminal-body {
  color: #f3eadb;
  font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
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

.hero-terminal-bar {
  justify-content: flex-start;
}

.hero-terminal-title {
  flex: 1;
}

.term-copy {
  align-items: center;
  background: rgb(255 255 255 / 6%);
  border: 1px solid rgb(255 255 255 / 10%);
  border-radius: 8px;
  color: rgba(243, 234, 219, 0.78);
  cursor: pointer;
  display: inline-flex;
  font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
  font-size: 11px;
  font-weight: 500;
  gap: 6px;
  height: 26px;
  letter-spacing: 0.04em;
  margin-right: -4px;
  padding: 0 10px;
  transition: background 0.15s, border-color 0.15s, color 0.15s;
}

.term-copy:hover {
  background: rgb(255 255 255 / 10%);
  border-color: rgb(255 255 255 / 16%);
  color: #f3eadb;
}

.term-copy:focus-visible {
  border-color: #a5b4fc;
  outline: none;
}

.term-copy[data-copied="true"] {
  background: rgb(52 211 153 / 14%);
  border-color: rgb(52 211 153 / 36%);
  color: #34d399;
}

.term-copy svg {
  flex-shrink: 0;
}

.term-copy-label {
  line-height: 1;
}

/* ============ Feature row ============ */

.feature-grid {
  display: grid;
  gap: 14px;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  margin: 0 auto;
  max-width: var(--max-width);
  width: 100%;
}

.feature-card {
  align-content: start;
}

/* ============ Size charts ============ */

.size-card {
  margin: 0 auto;
  max-width: var(--max-width);
  width: 100%;
}

.size-card-content {
  display: grid;
  gap: 18px;
}

.size-source {
  align-items: center;
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  padding-bottom: 14px;
  border-bottom: 1px solid var(--line);
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
  font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
  font-size: 11px;
  letter-spacing: 0.06em;
  text-transform: uppercase;
}

.size-bars {
  display: grid;
  gap: 14px;
}

.size-bars .tooltip {
  display: block;
}

.size-bars .tooltip-trigger {
  display: block;
  width: 100%;
}

.size-row {
  align-items: center;
  cursor: help;
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
  font-family: Inter, sans-serif;
  font-size: 14px;
  font-weight: 600;
  letter-spacing: 0;
}

.size-row-label span,
.size-row-value span {
  color: var(--ink-mute);
  font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
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
  margin: 0 auto;
  max-width: var(--max-width);
  width: 100%;
}

.card-editor {
  overflow: hidden;
  padding: 0;
  gap: 0;
}

.card-editor .select-list {
  max-height: 280px;
  overflow-y: auto;
  right: 0;
  left: auto;
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

/* ============ Demos ============ */

.demos-card {
  margin: 0 auto;
  max-width: var(--max-width);
  width: 100%;
}

.demos-card-content {
  padding-top: 0;
}

.demo-tab-label {
  align-items: center;
  display: inline-flex;
  gap: 10px;
}

.demo-pane {
  display: grid;
  gap: 22px;
  grid-template-columns: minmax(0, 0.85fr) minmax(0, 1.4fr);
  align-items: start;
}

.demo-pane-copy {
  align-content: center;
  display: grid;
  gap: 12px;
  padding: 4px 6px;
}

.demo-pane-title {
  color: var(--ink);
  font-family: Inter, sans-serif;
  font-size: clamp(22px, 2vw, 30px);
  font-weight: 600;
  letter-spacing: -0.02em;
  line-height: 1.15;
  margin: 0;
  max-width: 22ch;
}

.demo-pane-body {
  color: var(--ink-soft);
  font-family: Inter, sans-serif;
  font-size: 15px;
  line-height: 1.55;
  margin: 0;
  max-width: 56ch;
}

.demo-pane-frame {
  background: var(--code-bg);
  border: 1px solid var(--line);
  border-radius: var(--radius-inner);
  overflow: hidden;
}

.demo-pane-frame .card-bar {
  background: var(--card);
  border-bottom: 1px solid var(--line);
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

.doc-card {
  align-content: start;
}

.doc-head {
  align-items: center;
  display: flex;
  gap: 10px;
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
  font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
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
  background: var(--feature-bg-footer);
  border: 1px solid var(--feature-line);
  border-radius: var(--radius-card);
  box-shadow: var(--shadow-card);
  color: var(--feature-text);
  margin: 0 auto;
  max-width: var(--max-width);
  padding: 40px 40px 28px;
  width: 100%;
  display: flex;
  flex-direction: column;
  gap: 24px;
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
  color: var(--feature-bg-footer);
}

.footer-brand-name {
  color: var(--feature-text);
  font-family: Inter, sans-serif;
  font-size: 17px;
  font-weight: 600;
  letter-spacing: -0.01em;
}

.footer-tag {
  color: var(--feature-soft);
  font-family: Inter, sans-serif;
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
  font-family: Inter, sans-serif;
  font-size: 14px;
  font-weight: 500;
  transition: color 0.15s;
}

.footer-col a:hover {
  color: var(--feature-text);
}

.footer-meta {
  color: var(--feature-mute);
  font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
  font-size: 12px;
}

.footer-separator {
  margin: 0;
}

.footer-fineprint {
  color: var(--feature-mute);
  font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
  font-size: 12px;
  margin: 0;
}

/* ============ Responsive ============ */

@media (max-width: 1100px) {
  .hero-terminal-grid {
    gap: 36px;
    grid-template-columns: 1fr;
  }

  .feature-grid,
  .docs-grid {
    grid-template-columns: 1fr;
  }

  .demo-pane {
    grid-template-columns: 1fr;
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

  .topbar-tail {
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
}

.dxc {
  font-size: 14px;
  line-height: 1.55;
  overflow: auto;
}
"#;
