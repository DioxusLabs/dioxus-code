use std::collections::HashSet;

use dioxus::prelude::*;
use dioxus_code::{Code, CodeTheme, SourceCode, Theme, code};
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

const STARTER: &str = r##"use dioxus::prelude::*;
use dioxus_code::{Code, Theme, code};

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        Code {
            src: code!("/snippets/demo.rs"),
            theme: Theme::RUSTDOC_AYU,
        }
    }
}"##;

const PYTHON: &str = r#"def normalize(values):
    total = sum(values)
    return [round(value / total, 3) for value in values]

weights = normalize([12, 18, 7, 31])
print(weights)
"#;

const DOCS_INSTALL: &str = r#"dioxus-code = { version = "0.1", features = ["runtime"] }
"#;

const DOCS_RUNTIME: &str = r#"use dioxus_code::{Code, CodeTheme, SourceCode, Theme};

rsx! {
    Code {
        src: SourceCode::new(source).with_language("rust"),
        theme: CodeTheme::system(Theme::GITHUB_LIGHT, Theme::TOKYO_NIGHT),
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

const COMPONENTS_THEME_CSS: Asset = asset!("/assets/dx-components-theme.css");

const DEMO_FIT_SCRIPT: &str = r#"
const fitDemoCode = () => {
  document.querySelectorAll('.demo-pane-frame').forEach((frame) => {
    if (frame.offsetParent === null) return;
    const body = frame.querySelector('.card-code-body');
    const dxc = frame.querySelector('.dxc');
    if (!body || !dxc) return;
    const code = dxc.querySelector('code') || dxc;
    dxc.style.fontSize = '14px';
    const cs = window.getComputedStyle(dxc);
    const padX = parseFloat(cs.paddingLeft) + parseFloat(cs.paddingRight);
    const padY = parseFloat(cs.paddingTop) + parseFloat(cs.paddingBottom);
    const boxW = body.clientWidth - padX;
    const boxH = body.clientHeight - padY;
    const codeW = code.scrollWidth;
    const codeH = code.scrollHeight;
    if (boxW <= 0 || boxH <= 0 || codeW <= 0 || codeH <= 0) return;
    const scale = Math.min(boxW / codeW, boxH / codeH) * 0.94;
    const newSize = Math.max(11, Math.min(30, 14 * scale));
    dxc.style.fontSize = newSize + 'px';
  });
};
if (!window.__dxFitDemoSetup) {
  window.__dxFitDemoSetup = true;
  const ro = new ResizeObserver(() => fitDemoCode());
  const attach = () => {
    document.querySelectorAll('.demo-pane-frame .card-code-body').forEach((el) => ro.observe(el));
  };
  attach();
  new MutationObserver(() => { attach(); fitDemoCode(); })
    .observe(document.body, { childList: true, subtree: true });
  window.addEventListener('resize', fitDemoCode);
}
fitDemoCode();
requestAnimationFrame(fitDemoCode);
setTimeout(fitDemoCode, 120);
"#;

const DEMO_THEME_PAIRS: &[ThemePair] = &[
    ThemePair::new(Theme::ALABASTER, Theme::ZENBURN),
    ThemePair::new(Theme::AYU_LIGHT, Theme::AYU_DARK),
    ThemePair::new(Theme::CATPPUCCIN_LATTE, Theme::CATPPUCCIN_MOCHA),
    ThemePair::new(Theme::DAYFOX, Theme::TOKYO_NIGHT),
    ThemePair::new(Theme::GITHUB_LIGHT, Theme::GITHUB_DARK),
    ThemePair::new(Theme::GRUVBOX_LIGHT, Theme::GRUVBOX_DARK),
    ThemePair::new(Theme::LIGHT_OWL, Theme::DRACULA),
    ThemePair::new(Theme::LUCIUS_LIGHT, Theme::COBALT2),
    ThemePair::new(Theme::MELANGE_LIGHT, Theme::MELANGE_DARK),
    ThemePair::new(Theme::RUSTDOC_LIGHT, Theme::RUSTDOC_AYU),
    ThemePair::new(Theme::SOLARIZED_LIGHT, Theme::SOLARIZED_DARK),
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
    light: Theme,
    dark: Theme,
}

impl ThemePair {
    const fn new(light: Theme, dark: Theme) -> Self {
        Self { light, dark }
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
            Scheme::System => format!("{} / {}", self.light.name(), self.dark.name()),
            Scheme::Light => self.light.name().to_string(),
            Scheme::Dark => self.dark.name().to_string(),
        }
    }

    fn option_name(self) -> String {
        format!("{} / {}", self.light.name(), self.dark.name())
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
    let hero_theme_name = active_theme_pair.display_name(scheme_value);

    rsx! {
        document::Link { rel: "stylesheet", href: APP_CSS }
        main { class: "site-shell",
            Header { scheme }
            Hero { source: source(), theme: hero_theme, theme_name: hero_theme_name }
            FeatureRowReceipt {}
            Playground { source, active_theme, scheme: scheme_value }
            Demos { scheme: scheme_value }
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
                    value: "playground".to_string(),
                    to: "#playground",
                    "Playground"
                }
                NavbarItem {
                    index: 2usize,
                    value: "demos".to_string(),
                    to: "#demos",
                    "Demos"
                }
                NavbarItem {
                    index: 3usize,
                    value: "docs".to_string(),
                    to: "#docs",
                    "Docs"
                }
                NavbarNav { index: 4usize,
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
fn Hero(source: String, theme: CodeTheme, theme_name: String) -> Element {
    rsx! {
        section { id: "top", class: "hero hero-terminal",
            div { class: "hero-terminal-grid",
                div { class: "hero-terminal-copy",
                    div { class: "hero-eyebrow",
                        Badge { variant: BadgeVariant::Outline, "v0.1 · Dioxus 0.7" }
                    }
                    h1 { class: "hero-h1",
                        "Code highlighter for Dioxus; Runtime or "
                        em { "compile time" }
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
                }
                div { class: "hero-stage hero-stage-split",
                    div { class: "card-bar",
                        span { "src/counter.rs" }
                        span { "{theme_name}" }
                    }
                    div { class: "card-code-body",
                        Code { src: SourceCode::new(source).with_language("rust"), theme }
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
                    span { class: "receipt-eyebrow", "// what's in the box" }
                    h2 { class: "section-title receipt-title", "Itemized highlighting." }
                }
            }
            div { class: "receipt-stack",
                article { class: "receipt",
                    header { class: "receipt-head-stamp",
                        span { class: "receipt-stamp", "DIOXUS · CODE" }
                        span { class: "receipt-no", "RECEIPT #0042" }
                    }
                    div { class: "receipt-meta",
                        span { "ISSUED" }
                        span { "2026-05-05" }
                    }
                    div { class: "receipt-meta",
                        span { "RUNTIME" }
                        span { "DIOXUS 0.7" }
                    }
                    div { class: "receipt-rule" }
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
                    footer { class: "receipt-foot",
                        "** thank you for shipping with dioxus-code **"
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
                            p { class: "receipt-aside-text", "Pull it in when input is dynamic. Tree-sitter grammars detect language automatically." }
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
fn FeatureRowCodeBlock() -> Element {
    rsx! {
        section { class: "section",
            div { class: "section-head codeblock-head",
                div {
                    span { class: "codeblock-eyebrow", "// why.rs" }
                    h2 { class: "section-title codeblock-title",
                        "Two paths through the parser."
                    }
                }
            }
            div { class: "codeblock-card",
                header { class: "codeblock-bar",
                    span { class: "codeblock-dots",
                        i {}
                        i {}
                        i {}
                    }
                    span { class: "codeblock-file", "src/why.rs" }
                    span { class: "codeblock-lang", "rust" }
                }
                pre { class: "codeblock-body",
                    code { class: "codeblock-code",
                        span { class: "cb-line",
                            span { class: "cb-gut", " 1" }
                            span { class: "tok-doc", "//! Two paths through the parser." }
                        }
                        span { class: "cb-line",
                            span { class: "cb-gut", " 2" }
                            span { class: "tok-doc", "//!" }
                        }
                        span { class: "cb-line",
                            span { class: "cb-gut", " 3" }
                            span { class: "tok-doc", "//! `dioxus-code` ships compile-time markup and an" }
                        }
                        span { class: "cb-line",
                            span { class: "cb-gut", " 4" }
                            span { class: "tok-doc", "//! optional runtime engine. Both render through" }
                        }
                        span { class: "cb-line",
                            span { class: "cb-gut", " 5" }
                            span { class: "tok-doc", "//! the same " }
                            span { class: "tok-doc tok-doc-strong", "<Code/>" }
                            span { class: "tok-doc", " component." }
                        }
                        span { class: "cb-line cb-blank",
                            span { class: "cb-gut", " 6" }
                        }
                        span { class: "cb-line",
                            span { class: "cb-gut", " 7" }
                            span { class: "tok-doc", "/// Compile-time. The `code!` macro tokenizes during" }
                        }
                        span { class: "cb-line",
                            span { class: "cb-gut", " 8" }
                            span { class: "tok-doc", "/// `cargo build`, so the runtime gets pre-styled" }
                        }
                        span { class: "cb-line",
                            span { class: "cb-gut", " 9" }
                            span { class: "tok-doc", "/// markup with " }
                            span { class: "tok-doc tok-doc-strong", "no parser bytes." }
                        }
                        span { class: "cb-line",
                            span { class: "cb-gut", "10" }
                            span { class: "tok-attr", "#[macro_export]" }
                        }
                        span { class: "cb-line",
                            span { class: "cb-gut", "11" }
                            span { class: "tok-kw", "macro_rules! " }
                            span { class: "tok-fn", "code" }
                            span { class: "tok-pun", " {{ /* … */ }}" }
                        }
                        span { class: "cb-line cb-blank",
                            span { class: "cb-gut", "12" }
                        }
                        span { class: "cb-line",
                            span { class: "cb-gut", "13" }
                            span { class: "tok-doc", "/// Runtime. Pull `SourceCode` in when input is" }
                        }
                        span { class: "cb-line",
                            span { class: "cb-gut", "14" }
                            span { class: "tok-doc", "/// dynamic. Tree-sitter grammars detect language" }
                        }
                        span { class: "cb-line",
                            span { class: "cb-gut", "15" }
                            span { class: "tok-doc", "/// automatically." }
                        }
                        span { class: "cb-line",
                            span { class: "cb-gut", "16" }
                            span { class: "tok-kw", "pub struct " }
                            span { class: "tok-ty", "SourceCode" }
                            span { class: "tok-pun", " {{ /* … */ }}" }
                        }
                        span { class: "cb-line cb-blank",
                            span { class: "cb-gut", "17" }
                        }
                        span { class: "cb-line",
                            span { class: "cb-gut", "18" }
                            span { class: "tok-doc", "/// 30+ scoped themes — Tokyo Night, Catppuccin," }
                        }
                        span { class: "cb-line",
                            span { class: "cb-gut", "19" }
                            span { class: "tok-doc", "/// Dracula, Rosé Pine, GitHub… mix several on a" }
                        }
                        span { class: "cb-line",
                            span { class: "cb-gut", "20" }
                            span { class: "tok-doc", "/// single page." }
                        }
                        span { class: "cb-line",
                            span { class: "cb-gut", "21" }
                            span { class: "tok-kw", "pub enum " }
                            span { class: "tok-ty", "Theme" }
                            span { class: "tok-pun", " {{ /* 30+ */ }}" }
                        }
                    }
                }
                footer { class: "codeblock-foot",
                    span { class: "codeblock-foot-tag", "$" }
                    span { class: "codeblock-foot-cmd",
                        span { class: "tok-fn", "cargo" }
                        " add dioxus-code "
                        span { class: "tok-attr", "--features runtime" }
                    }
                }
            }
        }
    }
}

#[component]
fn FeatureRowPunchCard() -> Element {
    let cells_full: Vec<usize> = (0..28).collect();
    rsx! {
        section { class: "section",
            div { class: "section-head punchcard-head",
                div {
                    span { class: "punchcard-eyebrow", "// IBM 5081 · serial 0001" }
                    h2 { class: "section-title punchcard-title", "Punched at compile time." }
                }
            }
            div { class: "punchcard",
                header { class: "punchcard-bar",
                    span { class: "punchcard-tag", "5081" }
                    span { class: "punchcard-id", "DIOXUS-CODE / SERIAL 0001" }
                    span { class: "punchcard-rev", "REV 0.1 · CC" }
                }
                div { class: "punchcard-cols",
                    for n in [1, 5, 10, 15, 20, 25, 28] {
                        span { class: "punchcard-col", "{n}" }
                    }
                }
                ul { class: "punchcard-rows",
                    li { class: "punchcard-row",
                        span { class: "punchcard-label", "COMPILE-TIME" }
                        div { class: "punchcard-track",
                            for _ in cells_full.iter() {
                                span { class: "punchcard-cell solid" }
                            }
                        }
                        span { class: "punchcard-meta", "0 KB parser" }
                    }
                    li { class: "punchcard-row",
                        span { class: "punchcard-label", "RUNTIME" }
                        div { class: "punchcard-track",
                            for i in cells_full.iter() {
                                span { class: if matches!(*i, 4 | 11 | 19) { "punchcard-cell solid" } else { "punchcard-cell" } }
                            }
                        }
                        span { class: "punchcard-meta", "opt-in only" }
                    }
                    li { class: "punchcard-row",
                        span { class: "punchcard-label", "THEMES" }
                        div { class: "punchcard-track",
                            for _ in cells_full.iter() {
                                span { class: "punchcard-cell half" }
                            }
                        }
                        span { class: "punchcard-meta", "30+ scoped" }
                    }
                }
                footer { class: "punchcard-foot",
                    span { class: "punchcard-key-label", "KEY" }
                    span { class: "punchcard-key",
                        span { class: "punchcard-cell solid" }
                        "shipped"
                    }
                    span { class: "punchcard-key",
                        span { class: "punchcard-cell half" }
                        "scoped"
                    }
                    span { class: "punchcard-key",
                        span { class: "punchcard-cell" }
                        "opt-in"
                    }
                    span { class: "punchcard-pun", "PUNCHED ON cargo build" }
                }
            }
        }
    }
}

#[component]
fn FeatureRowSpecimen() -> Element {
    rsx! {
        section { class: "section",
            div { class: "section-head specimen-head",
                div {
                    span { class: "specimen-eyebrow", "// type specimen / parser weight" }
                    h2 { class: "section-title specimen-title", "0 bytes of parser." }
                }
            }
            div { class: "specimen",
                div { class: "specimen-stage",
                    span { class: "specimen-anno tl",
                        span { class: "specimen-anno-tick", "→" }
                        "what you ship"
                    }
                    span { class: "specimen-anno tr",
                        "in production"
                        span { class: "specimen-anno-tick", "←" }
                    }
                    div { class: "specimen-hero", "0" }
                    span { class: "specimen-unit", "KB · parser" }
                    span { class: "specimen-anno bl",
                        span { class: "specimen-anno-tick", "↑" }
                        "tree-sitter not included"
                    }
                    span { class: "specimen-anno br",
                        "themes scoped, not global"
                        span { class: "specimen-anno-tick", "↑" }
                    }
                }
                ol { class: "specimen-strip",
                    li { class: "specimen-sample",
                        span { class: "specimen-sample-label", "01 / macro" }
                        span { class: "specimen-sample-glyph", "code!" }
                        p { class: "specimen-sample-text",
                            "Tokenizes during cargo build. The runtime gets pre-styled markup."
                        }
                    }
                    li { class: "specimen-sample",
                        span { class: "specimen-sample-label", "02 / runtime" }
                        span { class: "specimen-sample-glyph", "SourceCode" }
                        p { class: "specimen-sample-text",
                            "Pull in when input is dynamic. Tree-sitter detects the language."
                        }
                    }
                    li { class: "specimen-sample",
                        span { class: "specimen-sample-label", "03 / themes" }
                        span { class: "specimen-sample-glyph", "Theme::*" }
                        p { class: "specimen-sample-text",
                            "30+ scoped variants — Tokyo Night, Catppuccin, Dracula, GitHub."
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn FeatureRowManifest() -> Element {
    rsx! {
        section { class: "section",
            div { class: "section-head manifest-head",
                div {
                    span { class: "manifest-eyebrow", "// bill of lading" }
                    h2 { class: "section-title", "What's actually in the box." }
                }
            }
            div { class: "manifest",
                header { class: "manifest-bar",
                    div { class: "manifest-bar-left",
                        span { class: "manifest-tag", "BILL OF LADING" }
                        span { class: "manifest-id", "DIOXUS-CODE / 0.1" }
                    }
                    span { class: "manifest-stamp", "SHIPPED" }
                }
                div { class: "manifest-meta",
                    div { class: "manifest-meta-cell",
                        span { class: "manifest-meta-key", "FROM" }
                        span { class: "manifest-meta-val", "cargo build" }
                    }
                    div { class: "manifest-meta-cell",
                        span { class: "manifest-meta-key", "TO" }
                        span { class: "manifest-meta-val", "your runtime" }
                    }
                    div { class: "manifest-meta-cell",
                        span { class: "manifest-meta-key", "DATE" }
                        span { class: "manifest-meta-val", "2026-05-05" }
                    }
                    div { class: "manifest-meta-cell",
                        span { class: "manifest-meta-key", "CARRIER" }
                        span { class: "manifest-meta-val", "<Code/>" }
                    }
                }
                table { class: "manifest-table",
                    thead {
                        tr {
                            th { class: "col-item", "ITEM" }
                            th { class: "col-qty", "QTY" }
                            th { class: "col-weight", "WEIGHT" }
                            th { class: "col-notes", "NOTES" }
                        }
                    }
                    tbody {
                        tr {
                            td { code { "code!()" } }
                            td { "1" }
                            td { class: "manifest-good", "0 KB" }
                            td { "compile-time markup" }
                        }
                        tr {
                            td { "Pre-styled spans" }
                            td { "—" }
                            td { class: "manifest-good", "0 KB" }
                            td { "baked at build" }
                        }
                        tr {
                            td { code { "<Code/>" }, " component" }
                            td { "1" }
                            td { "—" }
                            td { "renders both paths" }
                        }
                        tr {
                            td { "Themes (scoped)" }
                            td { "30+" }
                            td { "—" }
                            td { "Tokyo Night, Catppuccin, …" }
                        }
                        tr { class: "manifest-optional",
                            td { "SourceCode runtime" }
                            td { "opt-in" }
                            td { "+3.33 MiB" }
                            td { "behind feature flag" }
                        }
                    }
                }
                footer { class: "manifest-foot",
                    div { class: "manifest-sig",
                        span { class: "manifest-sig-key", "AUTHORIZED BY" }
                        span { class: "manifest-sig-val", "$ cargo build --release" }
                    }
                    div { class: "manifest-sig",
                        span { class: "manifest-sig-key", "SIGNATURE" }
                        span { class: "manifest-sig-line", "x" }
                    }
                }
            }
        }
    }
}

#[component]
fn FeatureRowBuildLog() -> Element {
    rsx! {
        section { class: "section",
            div { class: "section-head buildlog-head",
                div {
                    span { class: "buildlog-eyebrow", "$ cargo build --release" }
                    h2 { class: "section-title", "It compiles, then it ships." }
                }
            }
            div { class: "buildlog",
                header { class: "buildlog-bar",
                    span { class: "buildlog-dots",
                        i {}
                        i {}
                        i {}
                    }
                    span { class: "buildlog-title", "~/dioxus-code › cargo build --release" }
                    span { class: "buildlog-mode", "live" }
                }
                pre { class: "buildlog-body",
                    span { class: "log-line",
                        span { class: "log-status info", "[INFO]" }
                        " scanning source files for "
                        span { class: "log-tok", "code!()" }
                        " invocations"
                    }
                    span { class: "log-line",
                        span { class: "log-status ok", "[ OK ]" }
                        " tokenized "
                        span { class: "log-num", "17" }
                        " snippets at compile-time"
                    }
                    span { class: "log-line",
                        span { class: "log-status ok", "[ OK ]" }
                        " rendered pre-styled spans into binary"
                    }
                    span { class: "log-line",
                        span { class: "log-status skip", "[SKIP]" }
                        " runtime parser disabled (default)"
                    }
                    span { class: "log-line",
                        span { class: "log-status ok", "[ OK ]" }
                        " loaded "
                        span { class: "log-num", "31" }
                        " scoped themes"
                    }
                    span { class: "log-line",
                        span { class: "log-status info", "[INFO]" }
                        " Finished "
                        span { class: "log-tok", "`release`" }
                        " profile in "
                        span { class: "log-num", "4.21s" }
                    }
                    span { class: "log-line log-stat",
                        span { class: "log-status stat", "[STAT]" }
                        " parser bytes shipped: "
                        span { class: "log-big", "0" }
                    }
                    span { class: "log-line log-stat",
                        span { class: "log-status stat", "[STAT]" }
                        " delta over baseline: "
                        span { class: "log-big", "+0.20 MiB" }
                    }
                    span { class: "log-line log-prompt",
                        span { class: "log-prompt-mark", "↳" }
                        " ready"
                        span { class: "log-cursor" }
                    }
                }
            }
        }
    }
}

#[component]
fn FeatureRowRx() -> Element {
    rsx! {
        section { class: "section",
            div { class: "section-head rx-head",
                div {
                    span { class: "rx-eyebrow", "// prescription / take 1 daily" }
                    h2 { class: "section-title",
                        "Take one "
                        code { "<Code/>" }
                        " as needed."
                    }
                }
            }
            div { class: "rx",
                header { class: "rx-bar",
                    span { class: "rx-symbol", "℞" }
                    div { class: "rx-meta-block",
                        span { class: "rx-pharmacy", "DIOXUS · CODE PHARMACY" }
                        span { class: "rx-rxno", "Rx #0042 · 2026-05-05" }
                    }
                    span { class: "rx-controlled", "NO REFILL LIMIT" }
                }
                div { class: "rx-grid",
                    div { class: "rx-field",
                        span { class: "rx-field-key", "PATIENT" }
                        span { class: "rx-field-val", "any dioxus app" }
                    }
                    div { class: "rx-field",
                        span { class: "rx-field-key", "PRESCRIBER" }
                        span { class: "rx-field-val", "code!() macro" }
                    }
                    div { class: "rx-field",
                        span { class: "rx-field-key", "FILLED" }
                        span { class: "rx-field-val", "compile time" }
                    }
                    div { class: "rx-field",
                        span { class: "rx-field-key", "REFILLS" }
                        span { class: "rx-field-val", "∞" }
                    }
                }
                section { class: "rx-ingredients",
                    h3 { class: "rx-section-title", "ACTIVE INGREDIENTS" }
                    ul { class: "rx-ing-list",
                        li {
                            span { "code! macro" }
                            span { class: "rx-pct", "100%" }
                        }
                        li {
                            span { "Pre-styled spans" }
                            span { class: "rx-pct", "100%" }
                        }
                        li {
                            span { "Scoped themes" }
                            span { class: "rx-pct", "30+" }
                        }
                        li { class: "rx-zero",
                            span { "Parser bytes" }
                            span { class: "rx-pct", "0%" }
                        }
                    }
                }
                aside { class: "rx-warning",
                    span { class: "rx-warning-tag", "WARNING" }
                    "MAY CAUSE BINARIES TO LOSE WEIGHT WHEN SWITCHED FROM RUNTIME TO COMPILE-TIME. CONSULT YOUR CARGO BEFORE STOPPING."
                }
                footer { class: "rx-foot",
                    span { class: "rx-foot-dosage",
                        "DOSAGE: "
                        strong { "ONE <Code/> AS NEEDED" }
                    }
                    span { class: "rx-barcode" }
                }
            }
        }
    }
}

#[component]
fn FeatureRowDiff() -> Element {
    rsx! {
        section { class: "section",
            div { class: "section-head diff-head",
                div {
                    span { class: "diff-eyebrow", "// pull request · main ← compile-time" }
                    h2 { class: "section-title", "Two paths, one diff." }
                }
            }
            div { class: "diff",
                header { class: "diff-bar",
                    span { class: "diff-file diff-file-old", "before/highlight.rs" }
                    span { class: "diff-arrow", "→" }
                    span { class: "diff-file diff-file-new", "after/highlight.rs" }
                    span { class: "diff-stat-pill", "+5 / −3" }
                }
                pre { class: "diff-body",
                    span { class: "dl hunk", "@@ -1,8 +1,5 @@ pub fn render_snippet" }
                    span { class: "dl ctx", "  // a syntax highlighter" }
                    span { class: "dl del", "- use parser::{{Tokenizer, RuntimeParser}};" }
                    span { class: "dl del", "- let tokens = parser.parse(source);" }
                    span { class: "dl del", "- let html = render(tokens, theme);" }
                    span { class: "dl add", "+ use dioxus_code::{{code, Code, Theme}};" }
                    span { class: "dl add", "+ rsx! {{ Code {{ src: code!(\"src/main.rs\") }} }}" }
                    span { class: "dl ctx", "  // — same <Code/> handles dynamic input —" }
                    span { class: "dl hunk", "@@ binary @@" }
                    span { class: "dl del", "- parser bytes ............ +3.33 MiB" }
                    span { class: "dl del", "- runtime grammars ........ tree-sitter" }
                    span { class: "dl add", "+ parser bytes ............ 0 KB" }
                    span { class: "dl add", "+ themes scoped ........... 30+" }
                    span { class: "dl add", "+ runtime path ............ opt-in" }
                }
                footer { class: "diff-foot",
                    span { class: "diff-foot-stat add", "+5 added" }
                    span { class: "diff-foot-stat del", "−3 removed" }
                    span { class: "diff-foot-stat net", "net: −3.33 MiB" }
                    span { class: "diff-foot-action", "✓ approve & merge" }
                }
            }
        }
    }
}

#[component]
fn FeatureRowReceiptThermal() -> Element {
    rsx! {
        section { class: "section",
            div { class: "section-head receipt-head",
                div {
                    span { class: "receipt-eyebrow", "// the thermal slip" }
                    h2 { class: "section-title", "Itemized highlighting." }
                }
            }
            div { class: "rcpt-thermal-wrap",
                article { class: "rcpt-thermal",
                    header { class: "rcpt-th-head",
                        span { class: "rcpt-th-stamp", "DIOXUS · CODE" }
                        span { class: "rcpt-th-sub", "thermal printer · 80mm" }
                    }
                    div { class: "rcpt-th-meta",
                        span { "RX #" }
                        span { "0042 / 2026-05-05" }
                    }
                    div { class: "rcpt-th-meta",
                        span { "BUILD" }
                        span { "release · target/x86_64" }
                    }
                    div { class: "rcpt-th-rule" }
                    ul { class: "rcpt-th-items",
                        li {
                            span { "code! macro" }
                            span { class: "rcpt-th-dots" }
                            span { class: "rcpt-th-val", "INCLUDED" }
                        }
                        li {
                            span { "Pre-styled spans" }
                            span { class: "rcpt-th-dots" }
                            span { class: "rcpt-th-val", "INCLUDED" }
                        }
                        li {
                            span { "Themes (scoped)" }
                            span { class: "rcpt-th-dots" }
                            span { class: "rcpt-th-val", "× 30+" }
                        }
                        li { class: "rcpt-th-opt",
                            span { "SourceCode" }
                            span { class: "rcpt-th-dots" }
                            span { class: "rcpt-th-val", "OPT-IN" }
                        }
                        li { class: "rcpt-th-opt",
                            span { "tree-sitter" }
                            span { class: "rcpt-th-dots" }
                            span { class: "rcpt-th-val", "+3.33 MiB" }
                        }
                    }
                    div { class: "rcpt-th-rule double" }
                    div { class: "rcpt-th-total",
                        span { "PARSER BYTES" }
                        span { class: "rcpt-th-zero", "0" }
                    }
                    footer { class: "rcpt-th-foot",
                        "** thank you for shipping with dioxus-code **"
                    }
                    div { class: "rcpt-th-tear" }
                }
            }
        }
    }
}

#[component]
fn FeatureRowReceiptTape() -> Element {
    rsx! {
        section { class: "section",
            div { class: "section-head receipt-head",
                div {
                    span { class: "receipt-eyebrow", "// continuous tape · keep for your records" }
                    h2 { class: "section-title", "Three passes through cargo." }
                }
            }
            div { class: "rcpt-tape-wrap",
                article { class: "rcpt-tape",
                    div { class: "rcpt-tape-tear top" }
                    header { class: "rcpt-tape-head",
                        span { class: "rcpt-tape-stamp", "DIOXUS REGISTER" }
                        span { class: "rcpt-tape-no", "TAPE #0001 · 2026-05-05 · 14:21:08" }
                    }
                    section { class: "rcpt-tape-pass",
                        h3 { class: "rcpt-tape-passhead", "═══ PASS 1 · TOKENIZE ═══" }
                        ul { class: "rcpt-tape-rows",
                            li {
                                span { "scan source files" }
                                span { class: "rcpt-tape-val", "17 found" }
                            }
                            li {
                                span { "lex with tree-sitter" }
                                span { class: "rcpt-tape-val", "compile-time" }
                            }
                            li {
                                span { "emit token spans" }
                                span { class: "rcpt-tape-val", "✓" }
                            }
                        }
                        div { class: "rcpt-tape-sub",
                            span { "subtotal" }
                            span { class: "rcpt-tape-val", "0 KB shipped" }
                        }
                    }
                    section { class: "rcpt-tape-pass",
                        h3 { class: "rcpt-tape-passhead", "═══ PASS 2 · STYLE ═══" }
                        ul { class: "rcpt-tape-rows",
                            li {
                                span { "apply scoped theme" }
                                span { class: "rcpt-tape-val", "30+ avail." }
                            }
                            li {
                                span { "bake CSS variables" }
                                span { class: "rcpt-tape-val", "✓" }
                            }
                            li {
                                span { "inline pre-styled markup" }
                                span { class: "rcpt-tape-val", "✓" }
                            }
                        }
                        div { class: "rcpt-tape-sub",
                            span { "subtotal" }
                            span { class: "rcpt-tape-val", "0 KB shipped" }
                        }
                    }
                    section { class: "rcpt-tape-pass",
                        h3 { class: "rcpt-tape-passhead", "═══ PASS 3 · SHIP ═══" }
                        ul { class: "rcpt-tape-rows",
                            li {
                                span { "<Code/> render" }
                                span { class: "rcpt-tape-val", "static" }
                            }
                            li {
                                span { "runtime parser" }
                                span { class: "rcpt-tape-val rcpt-tape-skip", "skipped" }
                            }
                            li {
                                span { "binary delta" }
                                span { class: "rcpt-tape-val", "+0.20 MiB" }
                            }
                        }
                        div { class: "rcpt-tape-sub",
                            span { "subtotal" }
                            span { class: "rcpt-tape-val", "✓ ready" }
                        }
                    }
                    div { class: "rcpt-tape-rule double" }
                    div { class: "rcpt-tape-total",
                        span { "TOTAL · PARSER BYTES" }
                        span { class: "rcpt-tape-zero", "0" }
                    }
                    div { class: "rcpt-tape-rule" }
                    footer { class: "rcpt-tape-foot",
                        span { "$ cargo build --release" }
                        span { "Finished in 4.21s" }
                    }
                    div { class: "rcpt-tape-tear bottom" }
                }
            }
        }
    }
}

#[component]
fn FeatureRowReceiptCarbon() -> Element {
    rsx! {
        section { class: "section",
            div { class: "section-head receipt-head",
                div {
                    span { class: "receipt-eyebrow", "// original · duplicate · file copy" }
                    h2 { class: "section-title", "Itemized — in triplicate." }
                }
            }
            div { class: "rcpt-carbon-wrap",
                div { class: "rcpt-carbon-stack",
                    article { class: "rcpt-carbon copy-3",
                        header { class: "rcpt-cb-head",
                            span { class: "rcpt-cb-color-tag", "FILE" }
                            span { class: "rcpt-cb-stamp", "FILE COPY" }
                        }
                        div { class: "rcpt-cb-body",
                            div { class: "rcpt-cb-meta",
                                span { "DIOXUS · CODE" }
                                span { "Rx #0042" }
                            }
                            ul { class: "rcpt-cb-rows",
                                li {
                                    span { "code! macro" }
                                    span { "INCLUDED" }
                                }
                                li {
                                    span { "Themes" }
                                    span { "× 30+" }
                                }
                                li {
                                    span { "Parser bytes" }
                                    span { class: "rcpt-cb-zero", "0" }
                                }
                            }
                        }
                    }
                    article { class: "rcpt-carbon copy-2",
                        header { class: "rcpt-cb-head",
                            span { class: "rcpt-cb-color-tag", "DUPLICATE" }
                            span { class: "rcpt-cb-stamp", "DUPLICATE" }
                        }
                        div { class: "rcpt-cb-body",
                            div { class: "rcpt-cb-meta",
                                span { "DIOXUS · CODE" }
                                span { "Rx #0042" }
                            }
                            ul { class: "rcpt-cb-rows",
                                li {
                                    span { "code! macro" }
                                    span { "INCLUDED" }
                                }
                                li {
                                    span { "Themes" }
                                    span { "× 30+" }
                                }
                                li {
                                    span { "Parser bytes" }
                                    span { class: "rcpt-cb-zero", "0" }
                                }
                            }
                        }
                    }
                    article { class: "rcpt-carbon copy-1",
                        header { class: "rcpt-cb-head",
                            span { class: "rcpt-cb-color-tag", "ORIGINAL" }
                            span { class: "rcpt-cb-stamp", "ORIGINAL" }
                        }
                        div { class: "rcpt-cb-body",
                            div { class: "rcpt-cb-meta",
                                span { "DIOXUS · CODE" }
                                span { "Rx #0042 · 2026-05-05" }
                            }
                            ul { class: "rcpt-cb-rows",
                                li {
                                    span { "code! macro" }
                                    span { class: "rcpt-cb-val", "INCLUDED" }
                                }
                                li {
                                    span { "Pre-styled spans" }
                                    span { class: "rcpt-cb-val", "INCLUDED" }
                                }
                                li {
                                    span { "Themes (scoped)" }
                                    span { class: "rcpt-cb-val", "× 30+" }
                                }
                                li { class: "rcpt-cb-opt",
                                    span { "SourceCode runtime" }
                                    span { class: "rcpt-cb-val", "OPT-IN" }
                                }
                            }
                            div { class: "rcpt-cb-rule" }
                            div { class: "rcpt-cb-total",
                                span { "PARSER BYTES" }
                                span { class: "rcpt-cb-zero", "0" }
                            }
                            div { class: "rcpt-cb-stamp-overlay", "PAID" }
                        }
                    }
                }
                aside { class: "rcpt-carbon-legend",
                    div {
                        span { class: "rcpt-carbon-key copy-1" }
                        span { "ORIGINAL — what you ship" }
                    }
                    div {
                        span { class: "rcpt-carbon-key copy-2" }
                        span { "DUPLICATE — kept by the compiler" }
                    }
                    div {
                        span { class: "rcpt-carbon-key copy-3" }
                        span { "FILE — for your records" }
                    }
                }
            }
        }
    }
}

#[component]
fn FeatureRowReceiptInvoice() -> Element {
    rsx! {
        section { class: "section",
            div { class: "section-head receipt-head",
                div {
                    span { class: "receipt-eyebrow", "// invoice · paid in full" }
                    h2 { class: "section-title", "Paid at compile time." }
                }
            }
            article { class: "rcpt-inv",
                header { class: "rcpt-inv-letterhead",
                    div { class: "rcpt-inv-brand",
                        span { class: "rcpt-inv-mono", "DIOXUS" }
                        span { class: "rcpt-inv-brand-sub", "code · highlighter · co." }
                    }
                    div { class: "rcpt-inv-meta",
                        div {
                            span { class: "rcpt-inv-meta-key", "INVOICE №" }
                            span { class: "rcpt-inv-meta-val", "0042-CT" }
                        }
                        div {
                            span { class: "rcpt-inv-meta-key", "ISSUED" }
                            span { class: "rcpt-inv-meta-val", "2026-05-05" }
                        }
                        div {
                            span { class: "rcpt-inv-meta-key", "TERMS" }
                            span { class: "rcpt-inv-meta-val", "Net at build" }
                        }
                    }
                }
                section { class: "rcpt-inv-parties",
                    div { class: "rcpt-inv-party",
                        span { class: "rcpt-inv-party-key", "BILL TO" }
                        strong { "Your dioxus app" }
                        span { "/dist/index.html" }
                        span { "deployed runtime" }
                    }
                    div { class: "rcpt-inv-party",
                        span { class: "rcpt-inv-party-key", "FROM" }
                        strong { "$ cargo build --release" }
                        span { "target/x86_64-release" }
                        span { "buyer of last resort" }
                    }
                }
                table { class: "rcpt-inv-table",
                    thead {
                        tr {
                            th { class: "col-desc", "DESCRIPTION" }
                            th { class: "col-qty", "QTY" }
                            th { class: "col-unit", "UNIT" }
                            th { class: "col-amt", "AMOUNT" }
                        }
                    }
                    tbody {
                        tr {
                            td {
                                strong { "code! macro" }
                                br {}
                                span { class: "rcpt-inv-desc", "compile-time tokenization" }
                            }
                            td { "1" }
                            td { "0 KB" }
                            td { class: "rcpt-inv-zero", "0 KB" }
                        }
                        tr {
                            td {
                                strong { "Pre-styled span markup" }
                                br {}
                                span { class: "rcpt-inv-desc", "baked into binary at build" }
                            }
                            td { "—" }
                            td { "0 KB" }
                            td { class: "rcpt-inv-zero", "0 KB" }
                        }
                        tr {
                            td {
                                strong { "Scoped theme bundle" }
                                br {}
                                span { class: "rcpt-inv-desc", "Tokyo Night, Catppuccin, Dracula, …" }
                            }
                            td { "30+" }
                            td { "scoped CSS" }
                            td { "—" }
                        }
                        tr { class: "rcpt-inv-optional",
                            td {
                                strong { "SourceCode runtime" }
                                br {}
                                span { class: "rcpt-inv-desc", "tree-sitter, opt-in feature flag" }
                            }
                            td { "opt-in" }
                            td { "+3.33 MiB" }
                            td { "OPT-IN" }
                        }
                    }
                }
                section { class: "rcpt-inv-summary",
                    div { class: "rcpt-inv-summary-rows",
                        div { class: "rcpt-inv-row",
                            span { "Subtotal" }
                            span { "0 KB" }
                        }
                        div { class: "rcpt-inv-row",
                            span { "Runtime fees" }
                            span { "$0.00" }
                        }
                        div { class: "rcpt-inv-row total",
                            span { "TOTAL DUE" }
                            span { class: "rcpt-inv-total-zero", "0 KB" }
                        }
                    }
                    div { class: "rcpt-inv-stamp", "PAID IN FULL" }
                }
                footer { class: "rcpt-inv-foot",
                    "Terms: shipping is settled at "
                    code { "cargo build" }
                    ". Refunds available via "
                    code { "git revert" }
                    "."
                }
            }
        }
    }
}

#[component]
fn FeatureRowReceiptDiner() -> Element {
    rsx! {
        section { class: "section",
            div { class: "section-head receipt-head",
                div {
                    span { class: "receipt-eyebrow", "// the check, sir." }
                    h2 { class: "section-title", "Your bill from cargo." }
                }
            }
            article { class: "rcpt-diner",
                header { class: "rcpt-diner-head",
                    div { class: "rcpt-diner-brand",
                        span { class: "rcpt-diner-name", "DIOXUS · DINER" }
                        span { class: "rcpt-diner-tag", "open 24h · est. 2024" }
                    }
                    div { class: "rcpt-diner-meta",
                        div {
                            span { class: "rcpt-diner-meta-key", "TABLE" }
                            span { class: "rcpt-diner-meta-val", "0042" }
                        }
                        div {
                            span { class: "rcpt-diner-meta-key", "SERVER" }
                            span { class: "rcpt-diner-meta-val", "code!()" }
                        }
                        div {
                            span { class: "rcpt-diner-meta-key", "GUESTS" }
                            span { class: "rcpt-diner-meta-val", "1 app" }
                        }
                    }
                }
                ol { class: "rcpt-diner-items",
                    li {
                        span { class: "rcpt-diner-qty", "1" }
                        div { class: "rcpt-diner-desc",
                            strong { "code! macro" }
                            span { "compile-time, hot off the build" }
                        }
                        span { class: "rcpt-diner-price", "0 KB" }
                    }
                    li {
                        span { class: "rcpt-diner-qty", "1" }
                        div { class: "rcpt-diner-desc",
                            strong { "Pre-styled spans" }
                            span { "house specialty, baked in" }
                        }
                        span { class: "rcpt-diner-price", "0 KB" }
                    }
                    li {
                        span { class: "rcpt-diner-qty", "30+" }
                        div { class: "rcpt-diner-desc",
                            strong { "Themes (scoped CSS)" }
                            span { "Tokyo Night, Catppuccin, Dracula …" }
                        }
                        span { class: "rcpt-diner-price", "—" }
                    }
                    li { class: "rcpt-diner-side",
                        span { class: "rcpt-diner-qty", "side" }
                        div { class: "rcpt-diner-desc",
                            strong { "SourceCode runtime" }
                            span { "tree-sitter, only if you order it" }
                        }
                        span { class: "rcpt-diner-price", "+3.33 MiB" }
                    }
                }
                section { class: "rcpt-diner-tally",
                    div { class: "rcpt-diner-row",
                        span { "Subtotal (compile-time)" }
                        span { "0 KB" }
                    }
                    div { class: "rcpt-diner-row",
                        span { "Service charge" }
                        span { "$0.00" }
                    }
                    div { class: "rcpt-diner-row total",
                        span { "TOTAL · PARSER BYTES" }
                        span { class: "rcpt-diner-zero", "0" }
                    }
                }
                footer { class: "rcpt-diner-foot",
                    span { class: "rcpt-diner-thanks", "thanks, come back at next build" }
                    span { class: "rcpt-diner-tip",
                        span { class: "rcpt-diner-tip-key", "RECOMMENDED TIP" }
                        span { class: "rcpt-diner-tip-val", "★★★★★ on docs.rs" }
                    }
                }
            }
        }
    }
}

#[component]
fn FeatureRowReceiptAtm() -> Element {
    rsx! {
        section { class: "section",
            div { class: "section-head receipt-head",
                div {
                    span { class: "receipt-eyebrow", "// transaction complete" }
                    h2 { class: "section-title", "Withdrawn at compile time." }
                }
            }
            div { class: "rcpt-atm-wrap",
                article { class: "rcpt-atm",
                    header { class: "rcpt-atm-head",
                        span { class: "rcpt-atm-bank", "DIOXUS · BANK" }
                        span { class: "rcpt-atm-stamp", "TXN COMPLETE" }
                    }
                    div { class: "rcpt-atm-meta",
                        span { "ATM ID" }
                        span { "CARGO-001" }
                    }
                    div { class: "rcpt-atm-meta",
                        span { "DATE" }
                        span { "2026-05-05  14:21:08" }
                    }
                    div { class: "rcpt-atm-meta",
                        span { "ACCOUNT" }
                        span { "•••• 0042" }
                    }
                    div { class: "rcpt-atm-rule" }
                    div { class: "rcpt-atm-block",
                        span { class: "rcpt-atm-key", "TXN TYPE" }
                        span { class: "rcpt-atm-val", "WITHDRAWAL · COMPILE-TIME" }
                    }
                    div { class: "rcpt-atm-block",
                        span { class: "rcpt-atm-key", "WITHDRAWN" }
                        span { class: "rcpt-atm-val big", "0 KB" }
                        span { class: "rcpt-atm-sub", "(parser bytes — none requested)" }
                    }
                    div { class: "rcpt-atm-block",
                        span { class: "rcpt-atm-key", "DEPOSITED INTO BINARY" }
                        span { class: "rcpt-atm-val", "+0.20 MiB" }
                        span { class: "rcpt-atm-sub", "pre-styled markup, 30+ themes scoped" }
                    }
                    div { class: "rcpt-atm-rule" }
                    div { class: "rcpt-atm-balance",
                        span { class: "rcpt-atm-bal-key", "AVAILABLE BALANCE" }
                        span { class: "rcpt-atm-bal-val", "∞ snippets" }
                    }
                    div { class: "rcpt-atm-balance",
                        span { class: "rcpt-atm-bal-key", "RUNTIME PARSER OWED" }
                        span { class: "rcpt-atm-bal-val zero", "$0.00" }
                    }
                    footer { class: "rcpt-atm-foot",
                        span { class: "rcpt-atm-foot-line",
                            "PLEASE RETAIN FOR YOUR RECORDS — TRANSACTION ID 0042-CT-2026"
                        }
                        span { class: "rcpt-atm-foot-mag" }
                    }
                }
            }
        }
    }
}

#[component]
fn FeatureRowReceiptBoarding() -> Element {
    rsx! {
        section { class: "section",
            div { class: "section-head receipt-head",
                div {
                    span { class: "receipt-eyebrow", "// boarding pass · cargo airways" }
                    h2 { class: "section-title", "Now boarding: compile-time." }
                }
            }
            article { class: "rcpt-bp",
                section { class: "rcpt-bp-main",
                    header { class: "rcpt-bp-airline",
                        span { class: "rcpt-bp-mark", "✈" }
                        div {
                            span { class: "rcpt-bp-line", "CARGO AIRWAYS" }
                            span { class: "rcpt-bp-class", "BOARDING PASS · CT0042" }
                        }
                        span { class: "rcpt-bp-class-tag", "CLASS · COMPILE-TIME" }
                    }
                    div { class: "rcpt-bp-route",
                        div { class: "rcpt-bp-port",
                            span { class: "rcpt-bp-port-code", "SRC" }
                            span { class: "rcpt-bp-port-name", "your source files" }
                        }
                        span { class: "rcpt-bp-arrow", "──→" }
                        div { class: "rcpt-bp-port",
                            span { class: "rcpt-bp-port-code", "BIN" }
                            span { class: "rcpt-bp-port-name", "your binary" }
                        }
                    }
                    div { class: "rcpt-bp-grid",
                        div {
                            span { class: "rcpt-bp-key", "FLIGHT" }
                            span { class: "rcpt-bp-val", "code!()" }
                        }
                        div {
                            span { class: "rcpt-bp-key", "GATE" }
                            span { class: "rcpt-bp-val", "cargo build" }
                        }
                        div {
                            span { class: "rcpt-bp-key", "SEAT" }
                            span { class: "rcpt-bp-val", "0A" }
                        }
                        div {
                            span { class: "rcpt-bp-key", "DEPART" }
                            span { class: "rcpt-bp-val", "compile time" }
                        }
                        div {
                            span { class: "rcpt-bp-key", "ARRIVE" }
                            span { class: "rcpt-bp-val", "render time" }
                        }
                        div {
                            span { class: "rcpt-bp-key", "BAGGAGE" }
                            span { class: "rcpt-bp-val zero", "0 KB" }
                        }
                    }
                    div { class: "rcpt-bp-warn",
                        span { class: "rcpt-bp-warn-tag", "STANDBY" }
                        "SourceCode runtime parser — board only if explicitly upgraded (+3.33 MiB)"
                    }
                }
                aside { class: "rcpt-bp-stub",
                    span { class: "rcpt-bp-stub-tear" }
                    div { class: "rcpt-bp-stub-block",
                        span { class: "rcpt-bp-stub-key", "FLIGHT" }
                        span { class: "rcpt-bp-stub-val", "code!()" }
                    }
                    div { class: "rcpt-bp-stub-block",
                        span { class: "rcpt-bp-stub-key", "SEAT" }
                        span { class: "rcpt-bp-stub-val", "0A" }
                    }
                    div { class: "rcpt-bp-stub-block",
                        span { class: "rcpt-bp-stub-key", "GATE" }
                        span { class: "rcpt-bp-stub-val", "cargo" }
                    }
                    div { class: "rcpt-bp-stub-zero",
                        span { class: "rcpt-bp-stub-key", "PARSER" }
                        span { class: "rcpt-bp-stub-zero-val", "0 KB" }
                    }
                    div { class: "rcpt-bp-barcode" }
                }
            }
        }
    }
}

#[component]
fn FeatureRowReceiptVoid() -> Element {
    rsx! {
        section { class: "section",
            div { class: "section-head receipt-head",
                div {
                    span { class: "receipt-eyebrow", "// runtime line · voided" }
                    h2 { class: "section-title", "What you didn't pay for." }
                }
            }
            article { class: "rcpt-void",
                header { class: "rcpt-void-head",
                    span { class: "rcpt-void-stamp-mark", "VOID" }
                    div { class: "rcpt-void-meta",
                        span { class: "rcpt-void-meta-key", "RECEIPT" }
                        span { class: "rcpt-void-meta-val", "#0042 · 2026-05-05" }
                    }
                    div { class: "rcpt-void-meta",
                        span { class: "rcpt-void-meta-key", "CASHIER" }
                        span { class: "rcpt-void-meta-val", "cargo build --release" }
                    }
                }
                ul { class: "rcpt-void-items",
                    li { class: "rcpt-void-row keep",
                        span { class: "rcpt-void-mark", "✓" }
                        span { class: "rcpt-void-label", "code! macro" }
                        span { class: "rcpt-void-val", "INCLUDED" }
                    }
                    li { class: "rcpt-void-row keep",
                        span { class: "rcpt-void-mark", "✓" }
                        span { class: "rcpt-void-label", "Pre-styled spans" }
                        span { class: "rcpt-void-val", "INCLUDED" }
                    }
                    li { class: "rcpt-void-row keep",
                        span { class: "rcpt-void-mark", "✓" }
                        span { class: "rcpt-void-label", "Themes (scoped)" }
                        span { class: "rcpt-void-val", "× 30+" }
                    }
                    li { class: "rcpt-void-row voided",
                        span { class: "rcpt-void-mark void", "✕" }
                        span { class: "rcpt-void-label", "SourceCode runtime" }
                        span { class: "rcpt-void-val", "+3.33 MiB" }
                        span { class: "rcpt-void-strike" }
                    }
                    li { class: "rcpt-void-row voided",
                        span { class: "rcpt-void-mark void", "✕" }
                        span { class: "rcpt-void-label", "tree-sitter grammars" }
                        span { class: "rcpt-void-val", "DECLINED" }
                        span { class: "rcpt-void-strike" }
                    }
                }
                div { class: "rcpt-void-rule" }
                div { class: "rcpt-void-summary",
                    div {
                        span { class: "rcpt-void-summary-key", "Items kept" }
                        span { class: "rcpt-void-summary-val", "3" }
                    }
                    div {
                        span { class: "rcpt-void-summary-key", "Items voided" }
                        span { class: "rcpt-void-summary-val", "2" }
                    }
                    div { class: "rcpt-void-total",
                        span { class: "rcpt-void-total-key", "PARSER BYTES SHIPPED" }
                        span { class: "rcpt-void-total-val", "0" }
                    }
                }
                footer { class: "rcpt-void-foot",
                    "Voided lines remain on file. Re-enable with "
                    code { "--features runtime" }
                    "."
                }
                div { class: "rcpt-void-watermark", "VOID" }
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
    scheme: Scheme,
) -> Element {
    let theme_pairs = demo_theme_pairs();
    let active_idx = active_theme().min(theme_pairs.len() - 1);
    let theme_pair = theme_pairs[active_idx];
    let theme = theme_pair.code_theme(scheme);
    let theme_name = theme_pair.display_name(scheme);

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
                        span { "source.rs" }
                        span { class: "editor-meta",
                            span { "rust · " {format!("{} chars", source().chars().count())} }
                            span { class: "editor-meta-divider" }
                            span { "{theme_name}" }
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
fn Demos(scheme: Scheme) -> Element {
    let feature_pair = ThemePair::new(Theme::MELANGE_LIGHT, Theme::KANAGAWA_DRAGON);
    let runtime_pair = ThemePair::new(Theme::GITHUB_LIGHT, Theme::GITHUB_DARK);
    let feature_theme = feature_pair.code_theme(scheme);
    let runtime_theme = runtime_pair.code_theme(scheme);
    let feature_theme_name = feature_pair.display_name(scheme);
    let runtime_theme_name = runtime_pair.display_name(scheme);

    let mut active_demo = use_signal(|| "macro".to_string());
    let value = use_memo(move || Some(active_demo()));

    #[cfg(not(feature = "server"))]
    use_effect(move || {
        let _ = active_demo();
        document::eval(DEMO_FIT_SCRIPT);
    });

    rsx! {
        section { id: "demos", class: "section",
            div { class: "section-head",
                div {
                    h2 { class: "section-title", "Examples" }
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
                                        span { "{feature_theme_name}" }
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
                                        span { "{runtime_theme_name}" }
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
fn Docs(scheme: Scheme) -> Element {
    let theme_pairs = [
        ThemePair::new(Theme::MELANGE_LIGHT, Theme::MELANGE_DARK),
        ThemePair::new(Theme::RUSTDOC_LIGHT, Theme::RUSTDOC_AYU),
        ThemePair::new(Theme::GITHUB_LIGHT, Theme::TOKYO_NIGHT),
    ];
    let steps = doc_step_data();
    let themes = theme_pairs.map(|pair| pair.code_theme(scheme));
    let theme_names = theme_pairs.map(|pair| pair.display_name(scheme));

    rsx! {
        section { id: "docs", class: "section",
            div { class: "section-head",
                div {
                    h2 { class: "section-title", "Get started" }
                }
            }
            ol { class: "docs-timeline",
                for (i, step) in steps.iter().enumerate() {
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
                                    span { "{theme_names[i]}" }
                                }
                                div { class: "card-code-body",
                                    Code {
                                        src: SourceCode::new(step.code).with_language(step.language),
                                        theme: themes[i],
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
    language: &'static str,
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
            language: "toml",
            file_name: "Cargo.toml",
        },
        DocStepData {
            num: "02",
            eyebrow: "Runtime source",
            title: "SourceCode for live input",
            copy: "Pass any string through SourceCode. Provide a language hint when you already know it — Arborium handles tokenizing.",
            code: DOCS_RUNTIME,
            language: "rust",
            file_name: "runtime.rs",
        },
        DocStepData {
            num: "03",
            eyebrow: "Static source",
            title: "code! for snippets in your repo",
            copy: "Use the macro for examples, docs, and any source checked in alongside your app. Highlight markup is generated at compile time.",
            code: DOCS_STATIC,
            language: "rust",
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
                            span { class: "brand-mark", "dx" }
                            span { class: "footer-brand-name", "dioxus-code" }
                        }
                        p { class: "footer-tag",
                            "Syntax highlighting, designed for the inside of your Dioxus app."
                        }
                    }
                    div { class: "footer-col",
                        span { class: "card-eyebrow", "Project" }
                        a { href: "#features", "Why" }
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

fn demo_theme_pairs() -> &'static [ThemePair] {
    DEMO_THEME_PAIRS
}

const APP_CSS: Asset = asset!("/assets/app.css");
