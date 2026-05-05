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
        style { {APP_CSS} }
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
                        "Highlight code in Dioxus, at runtime or "
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

.hero-terminal {
  display: flex;
  flex-direction: column;
  min-height: calc(100dvh - 60px);
}

.hero-terminal .hero-terminal-grid {
  flex: 1 0 auto;
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
  margin-top: 32px;
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
  align-items: stretch;
  display: grid;
  gap: 56px;
  grid-template-columns: minmax(0, 1fr) minmax(0, 1.1fr);
  padding: 32px 0;
}

.hero-terminal-copy {
  display: flex;
  flex-direction: column;
}

.hero-terminal-block {
  background: var(--light, #0c0c0c) var(--dark, #161b22);
  border: 1px solid var(--light, transparent) var(--dark, var(--primary-color-6));
  border-radius: var(--radius-card);
  margin-top: auto;
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

/* ============ Variant banner (preview only) ============ */

.feature-variants {
  display: contents;
}

.variant-banner {
  align-items: baseline;
  background: var(--bg-tint);
  border-top: 1px dashed var(--line-strong);
  border-bottom: 1px dashed var(--line-strong);
  display: flex;
  font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
  font-size: 11px;
  gap: 14px;
  letter-spacing: 0.12em;
  margin: 56px 0 0;
  padding: 14px 28px;
  text-transform: uppercase;
}

.variant-banner-num {
  background: var(--ink);
  border-radius: 6px;
  color: var(--bg);
  font-weight: 600;
  letter-spacing: 0.08em;
  padding: 4px 8px;
}

.variant-banner-name {
  color: var(--ink-soft);
  font-weight: 500;
  letter-spacing: 0.06em;
  text-transform: none;
  font-size: 13px;
  font-family: Inter, sans-serif;
}

/* ============ Variant 02 — Ship vs Parse ledger ============ */

.ledger-head .ledger-eyebrow {
  color: var(--ink-mute);
  display: block;
  font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
  font-size: 12px;
  letter-spacing: 0.04em;
  margin-bottom: 10px;
}

.ledger-title em {
  color: var(--accent);
  font-style: italic;
  font-weight: 600;
}

.ledger-wrap {
  background: var(--card);
  border: 1px solid var(--line);
  border-radius: var(--radius-card);
  margin: 0 auto;
  max-width: var(--max-width);
  overflow: hidden;
  width: 100%;
}

.ledger-bar {
  align-items: center;
  background: var(--bg-tint);
  border-bottom: 1px solid var(--line);
  color: var(--ink-mute);
  display: flex;
  font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
  font-size: 11px;
  gap: 14px;
  letter-spacing: 0.14em;
  padding: 12px 22px;
  text-transform: uppercase;
}

.ledger-bar-tag {
  background: var(--ink);
  border-radius: 4px;
  color: var(--bg);
  font-weight: 600;
  letter-spacing: 0.1em;
  padding: 3px 8px;
}

.ledger-bar-file {
  color: var(--ink-soft);
  letter-spacing: 0;
  text-transform: none;
  font-size: 12px;
}

.ledger-bar-meta {
  margin-left: auto;
}

.ledger-grid {
  display: grid;
  grid-template-columns: minmax(0, 1fr) 1px minmax(0, 1fr);
}

.ledger-side {
  padding: 28px 28px 26px;
}

.ledger-side-head {
  display: grid;
  gap: 6px;
  margin-bottom: 22px;
}

.ledger-side-tag {
  color: var(--ink-mute);
  font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
  font-size: 11px;
  font-weight: 500;
  letter-spacing: 0.18em;
  text-transform: uppercase;
}

.ledger-ship .ledger-side-tag {
  color: var(--accent);
}

.ledger-side-title {
  color: var(--ink);
  font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
  font-size: clamp(24px, 2.4vw, 32px);
  font-weight: 500;
  letter-spacing: -0.02em;
  margin: 0;
}

.ledger-side-bang {
  color: var(--accent);
}

.ledger-side-sub {
  color: var(--ink-mute);
  font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
  font-size: 12px;
  letter-spacing: 0.04em;
  margin: 0;
}

.ledger-rows {
  display: grid;
  gap: 12px;
  list-style: none;
  margin: 0;
  padding: 0;
}

.ledger-row {
  align-items: baseline;
  display: grid;
  font-family: Inter, sans-serif;
  gap: 12px;
  grid-template-columns: 22px minmax(0, 1fr) auto;
  padding: 10px 0;
  border-bottom: 1px dashed var(--line);
}

.ledger-row:last-child {
  border-bottom: 0;
}

.ledger-mark {
  font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
  font-size: 14px;
  font-weight: 600;
}

.ledger-mark.ok {
  color: var(--accent);
}

.ledger-mark.add {
  color: var(--ink-mute);
}

.ledger-text {
  color: var(--ink);
  font-size: 14px;
  line-height: 1.5;
}

.ledger-amt {
  color: var(--ink-mute);
  font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
  font-size: 11px;
  font-weight: 500;
  letter-spacing: 0.06em;
  text-transform: uppercase;
  white-space: nowrap;
}

.ledger-divider {
  background: var(--line);
}

.ledger-foot {
  align-items: center;
  background: var(--bg-tint);
  border-top: 1px solid var(--line);
  color: var(--ink-soft);
  display: grid;
  font-family: Inter, sans-serif;
  font-size: 14px;
  gap: 18px;
  grid-template-columns: 90px minmax(0, 1fr);
  line-height: 1.55;
  padding: 18px 22px;
}

.ledger-foot-tag {
  color: var(--ink-mute);
  font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
  font-size: 11px;
  font-weight: 500;
  letter-spacing: 0.18em;
  text-transform: uppercase;
}

.ledger-foot-body strong {
  color: var(--ink);
  font-weight: 600;
}

/* ============ Variant 03 — itemized receipt ============ */

.receipt-head .receipt-eyebrow {
  color: var(--ink-mute);
  display: block;
  font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
  font-size: 12px;
  letter-spacing: 0.04em;
  margin-bottom: 10px;
}

.receipt-stack {
  display: grid;
  gap: 28px;
  grid-template-columns: minmax(380px, 480px) minmax(0, 1fr);
  margin: 0 auto;
  max-width: var(--max-width);
  width: 100%;
}

.receipt {
  background: var(--bg);
  border: 1px solid var(--line);
  box-shadow: var(--light, 0 26px 60px -36px rgb(0 0 0 / 30%)) var(--dark, 0 1px 0 0 rgb(255 255 255 / 4%) inset);
  color: var(--ink);
  display: grid;
  font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
  gap: 12px;
  padding: 36px 30px 32px;
  position: relative;
}

.receipt::before,
.receipt::after {
  background-image: linear-gradient(135deg, var(--bg) 25%, transparent 25.5%), linear-gradient(45deg, var(--bg) 25%, transparent 25.5%);
  background-position: top left, top left;
  background-repeat: repeat-x;
  background-size: 16px 12px;
  content: "";
  height: 12px;
  left: 0;
  position: absolute;
  right: 0;
}

.receipt::before {
  top: -1px;
  transform: rotate(180deg);
}

.receipt::after {
  bottom: -1px;
}

.receipt-head-stamp {
  align-items: baseline;
  display: flex;
  justify-content: space-between;
  letter-spacing: 0.18em;
  text-transform: uppercase;
}

.receipt-stamp {
  background: var(--ink);
  color: var(--bg);
  font-size: 10px;
  font-weight: 600;
  letter-spacing: 0.22em;
  padding: 5px 10px;
}

.receipt-no {
  color: var(--ink-mute);
  font-size: 10px;
  letter-spacing: 0.22em;
}

.receipt-meta {
  color: var(--ink-mute);
  display: flex;
  font-size: 11px;
  justify-content: space-between;
  letter-spacing: 0.14em;
  text-transform: uppercase;
}

.receipt-meta span:last-child {
  color: var(--ink);
}

.receipt-rule {
  border-top: 1px dashed var(--ink-mute);
  margin: 8px 0 4px;
}

.receipt-rule.double {
  border-top: 1px solid var(--ink);
  border-bottom: 1px solid var(--ink);
  height: 4px;
  margin: 12px 0 8px;
}

.receipt-items {
  display: grid;
  gap: 8px;
  list-style: none;
  margin: 0;
  padding: 0;
}

.receipt-item {
  align-items: baseline;
  display: grid;
  font-size: 12px;
  gap: 6px;
  grid-template-columns: minmax(0, auto) minmax(20px, 1fr) auto;
  letter-spacing: 0.04em;
}

.receipt-item.receipt-optional .receipt-label,
.receipt-item.receipt-optional .receipt-value {
  color: var(--ink-mute);
}

.receipt-label {
  color: var(--ink);
  text-transform: uppercase;
}

.receipt-dots {
  border-bottom: 1px dotted var(--ink-mute);
  height: 0;
  margin-bottom: 4px;
  min-width: 24px;
}

.receipt-value {
  color: var(--ink);
  font-weight: 600;
  letter-spacing: 0.08em;
  text-transform: uppercase;
  white-space: nowrap;
}

.receipt-total {
  align-items: baseline;
  display: flex;
  justify-content: space-between;
  margin-top: 4px;
}

.receipt-total-label {
  color: var(--ink);
  font-size: 12px;
  font-weight: 600;
  letter-spacing: 0.16em;
  text-transform: uppercase;
}

.receipt-total-value {
  color: var(--accent);
  font-size: clamp(28px, 3vw, 40px);
  font-weight: 600;
  letter-spacing: -0.02em;
}

.receipt-foot {
  color: var(--ink-mute);
  font-size: 11px;
  letter-spacing: 0.18em;
  margin-top: 6px;
  text-align: center;
  text-transform: uppercase;
}

.receipt-aside {
  align-self: center;
  display: grid;
  gap: 18px;
}

.receipt-aside-row {
  align-items: baseline;
  border-top: 1px solid var(--line);
  display: grid;
  gap: 18px;
  grid-template-columns: 56px minmax(0, 1fr);
  padding: 18px 4px 4px;
}

.receipt-aside-row:first-child {
  border-top: 0;
  padding-top: 4px;
}

.receipt-aside-num {
  color: var(--ink-mute);
  font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
  font-size: 28px;
  font-weight: 500;
  letter-spacing: -0.02em;
}

.receipt-aside-title {
  color: var(--ink);
  font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
  font-size: 18px;
  font-weight: 500;
  letter-spacing: -0.01em;
  margin: 0 0 6px;
}

.receipt-aside-text {
  color: var(--ink-soft);
  font-family: Inter, sans-serif;
  font-size: 14px;
  line-height: 1.55;
  margin: 0;
  max-width: 42ch;
}

/* ============ Variant 04 — section as code ============ */

.codeblock-head .codeblock-eyebrow {
  color: var(--ink-mute);
  display: block;
  font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
  font-size: 12px;
  letter-spacing: 0.04em;
  margin-bottom: 10px;
}

.codeblock-title {
  font-family: ui-monospace, SFMono-Regular, Menlo, monospace !important;
  font-size: clamp(26px, 3.2vw, 40px) !important;
  font-weight: 500 !important;
  letter-spacing: -0.02em !important;
}

.codeblock-card {
  background: var(--card);
  border: 1px solid var(--line);
  border-radius: var(--radius-card);
  box-shadow: var(--shadow-card);
  margin: 0 auto;
  max-width: var(--max-width);
  overflow: hidden;
  width: 100%;
}

.codeblock-bar {
  align-items: center;
  background: var(--bg-tint);
  border-bottom: 1px solid var(--line);
  color: var(--ink-mute);
  display: flex;
  font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
  font-size: 11px;
  gap: 14px;
  letter-spacing: 0.08em;
  padding: 14px 22px;
}

.codeblock-dots {
  display: inline-flex;
  gap: 6px;
}

.codeblock-dots i {
  background: var(--line-strong);
  border-radius: 50%;
  display: inline-block;
  height: 10px;
  width: 10px;
}

.codeblock-dots i:nth-child(1) { background: #ef4444; }
.codeblock-dots i:nth-child(2) { background: #f59e0b; }
.codeblock-dots i:nth-child(3) { background: #10b981; }

.codeblock-file {
  color: var(--ink);
  font-weight: 500;
  letter-spacing: 0.02em;
}

.codeblock-lang {
  color: var(--ink-mute);
  margin-left: auto;
  text-transform: uppercase;
  letter-spacing: 0.18em;
}

.codeblock-body {
  background: var(--code-bg);
  font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
  font-size: 13px;
  line-height: 1.7;
  margin: 0;
  overflow-x: auto;
  padding: 22px 0;
}

.codeblock-code {
  display: block;
  font-family: inherit;
}

.cb-line {
  display: grid;
  grid-template-columns: 56px minmax(0, 1fr);
  padding: 0 24px 0 0;
}

.cb-blank {
  height: 1.7em;
}

.cb-gut {
  color: var(--ink-mute);
  opacity: 0.5;
  padding-right: 18px;
  text-align: right;
  user-select: none;
}

.codeblock-body .tok-doc {
  color: var(--ink-mute);
  font-style: italic;
}

.codeblock-body .tok-doc-strong {
  color: var(--ink);
  font-style: normal;
  font-weight: 600;
}

.codeblock-body .tok-attr {
  color: var(--accent);
}

.codeblock-body .tok-kw {
  color: var(--accent);
  font-weight: 500;
}

.codeblock-body .tok-ty {
  color: var(--ink);
  font-weight: 500;
}

.codeblock-body .tok-fn {
  color: var(--ink);
  font-weight: 500;
}

.codeblock-body .tok-pun {
  color: var(--ink-soft);
}

.codeblock-foot {
  align-items: center;
  background: var(--bg-tint);
  border-top: 1px solid var(--line);
  color: var(--ink-soft);
  display: flex;
  font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
  font-size: 13px;
  gap: 14px;
  padding: 14px 22px;
}

.codeblock-foot-tag {
  color: var(--ink-mute);
  font-weight: 500;
}

.codeblock-foot-cmd .tok-fn {
  color: var(--ink);
}

.codeblock-foot-cmd .tok-attr {
  color: var(--accent);
}

/* ============ Variant 05 — punch card ============ */

.punchcard-head .punchcard-eyebrow,
.specimen-head .specimen-eyebrow,
.manifest-head .manifest-eyebrow,
.buildlog-head .buildlog-eyebrow,
.rx-head .rx-eyebrow,
.diff-head .diff-eyebrow {
  color: var(--ink-mute);
  display: block;
  font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
  font-size: 12px;
  letter-spacing: 0.04em;
  margin-bottom: 10px;
}

.punchcard {
  background: var(--light, #f3ead8) var(--dark, #1a1814);
  border: 1px solid var(--line);
  border-radius: 6px;
  color: var(--light, #2d2418) var(--dark, #d8c8a8);
  font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
  margin: 0 auto;
  max-width: var(--max-width);
  padding: 22px 26px 24px;
  position: relative;
  width: 100%;
}

.punchcard::before {
  background: linear-gradient(180deg, currentColor 0 100%);
  content: "";
  height: 60px;
  left: 0;
  opacity: 0.12;
  position: absolute;
  top: 0;
  width: 28px;
  clip-path: polygon(0 0, 100% 0, 60% 100%, 0 100%);
}

.punchcard-bar {
  align-items: baseline;
  border-bottom: 1px solid currentColor;
  display: flex;
  font-size: 11px;
  gap: 16px;
  letter-spacing: 0.16em;
  opacity: 0.85;
  padding: 0 0 12px;
  text-transform: uppercase;
}

.punchcard-tag {
  border: 1px solid currentColor;
  font-weight: 600;
  letter-spacing: 0.18em;
  padding: 3px 8px;
}

.punchcard-id {
  flex: 1;
}

.punchcard-rev {
  opacity: 0.7;
}

.punchcard-cols {
  display: grid;
  font-size: 10px;
  grid-template-columns: 130px minmax(0, 1fr) 110px;
  letter-spacing: 0.1em;
  margin-top: 14px;
  opacity: 0.5;
}

.punchcard-cols::after,
.punchcard-cols::before {
  content: "";
}

.punchcard-cols span {
  display: contents;
}

.punchcard-cols {
  display: flex;
  gap: 0;
  justify-content: space-between;
  padding: 8px 130px 0 130px;
}

.punchcard-col {
  font-size: 10px;
  font-weight: 500;
  letter-spacing: 0.06em;
  opacity: 0.45;
}

.punchcard-rows {
  display: grid;
  gap: 6px;
  list-style: none;
  margin: 8px 0 16px;
  padding: 0;
}

.punchcard-row {
  align-items: center;
  display: grid;
  gap: 18px;
  grid-template-columns: 130px minmax(0, 1fr) 110px;
}

.punchcard-label {
  font-size: 11px;
  font-weight: 600;
  letter-spacing: 0.18em;
  text-transform: uppercase;
}

.punchcard-track {
  display: flex;
  gap: 4px;
}

.punchcard-cell {
  border: 1px solid currentColor;
  border-radius: 1px;
  flex: 1;
  height: 22px;
  opacity: 0.5;
}

.punchcard-cell.solid {
  background: currentColor;
  opacity: 1;
}

.punchcard-cell.half {
  background: repeating-linear-gradient(135deg, currentColor 0 2px, transparent 2px 5px);
  opacity: 0.85;
}

.punchcard-meta {
  font-size: 11px;
  letter-spacing: 0.06em;
  opacity: 0.75;
  text-align: right;
  text-transform: uppercase;
}

.punchcard-foot {
  align-items: center;
  border-top: 1px dashed currentColor;
  display: flex;
  flex-wrap: wrap;
  font-size: 11px;
  gap: 16px;
  letter-spacing: 0.12em;
  margin-top: 10px;
  opacity: 0.85;
  padding-top: 12px;
  text-transform: uppercase;
}

.punchcard-key-label {
  font-weight: 600;
  letter-spacing: 0.2em;
  opacity: 0.7;
}

.punchcard-key {
  align-items: center;
  display: inline-flex;
  gap: 8px;
}

.punchcard-key .punchcard-cell {
  flex: 0 0 18px;
  height: 14px;
}

.punchcard-pun {
  margin-left: auto;
  opacity: 0.6;
}

/* ============ Variant 06 — type specimen ============ */

.specimen {
  margin: 0 auto;
  max-width: var(--max-width);
  width: 100%;
}

.specimen-stage {
  background: var(--bg);
  border: 1px solid var(--line);
  border-radius: var(--radius-card);
  display: grid;
  grid-template-areas:
    "tl   .    tr"
    ".    hero .   "
    "bl   .    br";
  grid-template-columns: minmax(140px, 1fr) auto minmax(140px, 1fr);
  grid-template-rows: auto auto auto;
  padding: 56px 36px 40px;
  position: relative;
  text-align: center;
}

.specimen-anno {
  align-self: start;
  color: var(--ink-mute);
  display: inline-flex;
  font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
  font-size: 11px;
  gap: 6px;
  letter-spacing: 0.08em;
  text-transform: uppercase;
}

.specimen-anno.tl { grid-area: tl; justify-self: start; }
.specimen-anno.tr { grid-area: tr; justify-self: end; }
.specimen-anno.bl { grid-area: bl; justify-self: start; align-self: end; }
.specimen-anno.br { grid-area: br; justify-self: end; align-self: end; }

.specimen-anno-tick {
  color: var(--accent);
  font-weight: 600;
}

.specimen-hero {
  color: var(--ink);
  font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
  font-size: clamp(120px, 22vw, 280px);
  font-weight: 400;
  grid-area: hero;
  letter-spacing: -0.06em;
  line-height: 0.85;
  margin: 0;
  position: relative;
}

.specimen-hero::before,
.specimen-hero::after {
  background: var(--line);
  content: "";
  height: 1px;
  position: absolute;
  top: 50%;
  width: 64px;
}

.specimen-hero::before { left: -84px; }
.specimen-hero::after { right: -84px; }

.specimen-unit {
  color: var(--ink-mute);
  font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
  font-size: 12px;
  grid-column: 1 / -1;
  grid-row: 3;
  justify-self: center;
  letter-spacing: 0.18em;
  margin-top: 4px;
  padding-top: 12px;
  text-transform: uppercase;
}

.specimen-strip {
  border-top: 1px solid var(--line);
  display: grid;
  gap: 0;
  grid-template-columns: repeat(3, 1fr);
  list-style: none;
  margin: 0;
  padding: 0;
}

.specimen-sample {
  border-right: 1px solid var(--line);
  display: grid;
  gap: 10px;
  padding: 24px 22px;
}

.specimen-sample:last-child {
  border-right: 0;
}

.specimen-sample-label {
  color: var(--ink-mute);
  font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
  font-size: 11px;
  letter-spacing: 0.18em;
  text-transform: uppercase;
}

.specimen-sample-glyph {
  color: var(--ink);
  font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
  font-size: clamp(28px, 3.4vw, 44px);
  font-weight: 500;
  letter-spacing: -0.025em;
  line-height: 1;
}

.specimen-sample-text {
  color: var(--ink-soft);
  font-family: Inter, sans-serif;
  font-size: 14px;
  line-height: 1.55;
  margin: 0;
}

/* ============ Variant 07 — bill of lading ============ */

.manifest {
  background: var(--bg);
  border: 1px solid var(--line);
  border-radius: 4px;
  box-shadow: var(--light, 0 26px 60px -36px rgb(0 0 0 / 30%)) var(--dark, 0 1px 0 0 rgb(255 255 255 / 4%) inset);
  font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
  margin: 0 auto;
  max-width: var(--max-width);
  overflow: hidden;
  position: relative;
  width: 100%;
}

.manifest-bar {
  align-items: center;
  background: var(--ink);
  color: var(--bg);
  display: flex;
  gap: 24px;
  justify-content: space-between;
  padding: 16px 26px;
}

.manifest-bar-left {
  align-items: baseline;
  display: flex;
  gap: 18px;
}

.manifest-tag {
  font-size: 12px;
  font-weight: 600;
  letter-spacing: 0.22em;
}

.manifest-id {
  font-size: 11px;
  letter-spacing: 0.1em;
  opacity: 0.7;
}

.manifest-stamp {
  border: 2px solid #dc2626;
  color: #dc2626;
  font-size: 14px;
  font-weight: 700;
  letter-spacing: 0.28em;
  padding: 6px 14px;
  transform: rotate(-3deg);
}

.manifest-meta {
  border-bottom: 1px solid var(--line);
  display: grid;
  grid-template-columns: repeat(4, minmax(0, 1fr));
}

.manifest-meta-cell {
  border-right: 1px solid var(--line);
  display: grid;
  gap: 4px;
  padding: 14px 18px;
}

.manifest-meta-cell:last-child {
  border-right: 0;
}

.manifest-meta-key {
  color: var(--ink-mute);
  font-size: 10px;
  letter-spacing: 0.2em;
  text-transform: uppercase;
}

.manifest-meta-val {
  color: var(--ink);
  font-size: 14px;
  font-weight: 500;
}

.manifest-table {
  border-collapse: collapse;
  font-size: 13px;
  width: 100%;
}

.manifest-table th,
.manifest-table td {
  border-bottom: 1px solid var(--line);
  padding: 12px 18px;
  text-align: left;
}

.manifest-table th {
  background: var(--bg-tint);
  color: var(--ink-mute);
  font-size: 11px;
  font-weight: 600;
  letter-spacing: 0.16em;
  text-transform: uppercase;
}

.manifest-table .col-qty,
.manifest-table .col-weight {
  width: 110px;
}

.manifest-table tbody td {
  color: var(--ink);
}

.manifest-table tbody code {
  background: var(--bg-tint);
  border: 1px solid var(--line);
  border-radius: 4px;
  font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
  font-size: 12px;
  padding: 1px 6px;
}

.manifest-good {
  color: #16a34a !important;
  font-weight: 600;
}

.manifest-optional {
  background: var(--bg-tint);
  color: var(--ink-mute) !important;
}

.manifest-optional td {
  color: var(--ink-mute);
}

.manifest-foot {
  align-items: end;
  background: var(--bg-tint);
  border-top: 1px solid var(--line);
  display: grid;
  font-size: 12px;
  gap: 32px;
  grid-template-columns: minmax(0, 1.4fr) minmax(0, 1fr);
  padding: 22px 26px 26px;
}

.manifest-sig {
  display: grid;
  gap: 6px;
}

.manifest-sig-key {
  color: var(--ink-mute);
  font-size: 10px;
  letter-spacing: 0.2em;
  text-transform: uppercase;
}

.manifest-sig-val {
  color: var(--ink);
  font-weight: 500;
}

.manifest-sig-line {
  border-bottom: 2px solid var(--ink);
  color: var(--ink);
  font-family: "Brush Script MT", "Comic Sans MS", cursive;
  font-size: 22px;
  padding-bottom: 4px;
}

/* ============ Variant 08 — terminal build log ============ */

.buildlog {
  background: #0d0e12;
  border: 1px solid var(--line);
  border-radius: var(--radius-card);
  color: #d8dee9;
  font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
  margin: 0 auto;
  max-width: var(--max-width);
  overflow: hidden;
  width: 100%;
}

.buildlog-bar {
  align-items: center;
  background: #15171c;
  border-bottom: 1px solid #2a2d36;
  color: #8b94a7;
  display: flex;
  font-size: 12px;
  gap: 14px;
  padding: 12px 18px;
}

.buildlog-dots {
  display: inline-flex;
  gap: 6px;
}

.buildlog-dots i {
  border-radius: 50%;
  display: inline-block;
  height: 10px;
  width: 10px;
}

.buildlog-dots i:nth-child(1) { background: #ef4444; }
.buildlog-dots i:nth-child(2) { background: #f59e0b; }
.buildlog-dots i:nth-child(3) { background: #10b981; }

.buildlog-title {
  color: #d8dee9;
  font-weight: 500;
}

.buildlog-mode {
  background: #142a21;
  border: 1px solid #1f4a37;
  border-radius: 4px;
  color: #4ade80;
  font-size: 10px;
  letter-spacing: 0.18em;
  margin-left: auto;
  padding: 3px 8px;
  text-transform: uppercase;
}

.buildlog-body {
  font-family: inherit;
  font-size: 13px;
  line-height: 1.85;
  margin: 0;
  padding: 22px 24px 26px;
}

.log-line {
  display: block;
}

.log-status {
  display: inline-block;
  font-weight: 700;
  margin-right: 10px;
  width: 56px;
}

.log-status.ok { color: #4ade80; }
.log-status.skip { color: #fbbf24; }
.log-status.info { color: #60a5fa; }
.log-status.stat { color: #a78bfa; }

.log-tok {
  color: #f9b97f;
}

.log-num {
  color: #fde68a;
}

.log-stat .log-big {
  color: #ffffff;
  font-weight: 600;
}

.log-prompt {
  color: #4ade80;
  margin-top: 14px;
}

.log-prompt-mark {
  margin-right: 6px;
}

.log-cursor {
  background: #4ade80;
  display: inline-block;
  height: 14px;
  margin-left: 6px;
  vertical-align: -2px;
  width: 8px;
  animation: log-blink 1.1s steps(2, end) infinite;
}

@keyframes log-blink {
  to { opacity: 0; }
}

/* ============ Variant 09 — Rx prescription label ============ */

.rx {
  background: var(--bg);
  border: 1px solid var(--line);
  border-radius: 6px;
  box-shadow: var(--shadow-card);
  font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
  margin: 0 auto;
  max-width: 720px;
  overflow: hidden;
  width: 100%;
}

.rx-bar {
  align-items: center;
  background: linear-gradient(0deg, var(--bg) 0%, var(--bg) 60%, var(--accent) 60%, var(--accent) 100%);
  border-bottom: 2px solid var(--accent);
  color: var(--ink);
  display: grid;
  gap: 14px;
  grid-template-columns: 64px minmax(0, 1fr) auto;
  padding: 14px 22px 18px;
}

.rx-symbol {
  background: var(--bg);
  border: 2px solid var(--accent);
  border-radius: 6px;
  color: var(--accent);
  font-family: Georgia, "Times New Roman", serif;
  font-size: 36px;
  font-weight: 700;
  height: 56px;
  line-height: 1;
  padding-top: 8px;
  text-align: center;
  width: 56px;
}

.rx-meta-block {
  display: grid;
  gap: 4px;
}

.rx-pharmacy {
  color: var(--ink);
  font-size: 13px;
  font-weight: 700;
  letter-spacing: 0.18em;
}

.rx-rxno {
  color: var(--ink-mute);
  font-size: 11px;
  letter-spacing: 0.08em;
}

.rx-controlled {
  background: #fef3c7;
  border: 1px solid #f59e0b;
  border-radius: 4px;
  color: #92400e;
  font-size: 10px;
  font-weight: 700;
  letter-spacing: 0.18em;
  padding: 5px 10px;
  text-transform: uppercase;
}

.rx-grid {
  border-bottom: 1px dashed var(--line-strong);
  display: grid;
  grid-template-columns: repeat(4, minmax(0, 1fr));
}

.rx-field {
  border-right: 1px dashed var(--line-strong);
  display: grid;
  gap: 4px;
  padding: 14px 16px;
}

.rx-field:last-child {
  border-right: 0;
}

.rx-field-key {
  color: var(--ink-mute);
  font-size: 10px;
  letter-spacing: 0.18em;
  text-transform: uppercase;
}

.rx-field-val {
  color: var(--ink);
  font-size: 14px;
  font-weight: 500;
}

.rx-ingredients {
  padding: 18px 22px 12px;
}

.rx-section-title {
  color: var(--ink);
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 0.22em;
  margin: 0 0 10px;
  text-transform: uppercase;
}

.rx-ing-list {
  display: grid;
  gap: 6px;
  list-style: none;
  margin: 0;
  padding: 0;
}

.rx-ing-list li {
  align-items: baseline;
  border-bottom: 1px dotted var(--line-strong);
  color: var(--ink);
  display: flex;
  font-size: 13px;
  justify-content: space-between;
  padding: 6px 0;
}

.rx-ing-list li.rx-zero {
  background: linear-gradient(90deg, transparent, rgb(34 197 94 / 12%), transparent);
}

.rx-pct {
  color: var(--ink-mute);
  font-weight: 600;
  letter-spacing: 0.06em;
}

.rx-zero .rx-pct {
  color: #16a34a;
}

.rx-warning {
  background: #fef9c3;
  border-bottom: 1px solid var(--line);
  border-top: 3px solid #f59e0b;
  color: #713f12;
  font-size: 12px;
  letter-spacing: 0.04em;
  line-height: 1.5;
  margin: 8px 0 0;
  padding: 14px 22px;
  text-transform: uppercase;
}

.rx-warning-tag {
  background: #f59e0b;
  border-radius: 3px;
  color: #1f2937;
  font-weight: 700;
  letter-spacing: 0.2em;
  margin-right: 10px;
  padding: 3px 8px;
}

.rx-foot {
  align-items: center;
  background: var(--bg-tint);
  border-top: 1px solid var(--line);
  color: var(--ink);
  display: flex;
  font-size: 11px;
  gap: 18px;
  justify-content: space-between;
  letter-spacing: 0.16em;
  padding: 14px 22px;
  text-transform: uppercase;
}

.rx-foot-dosage strong {
  color: var(--accent);
}

.rx-barcode {
  background: repeating-linear-gradient(
    90deg,
    var(--ink) 0 2px,
    transparent 2px 4px,
    var(--ink) 4px 5px,
    transparent 5px 8px,
    var(--ink) 8px 11px,
    transparent 11px 13px,
    var(--ink) 13px 14px,
    transparent 14px 17px
  );
  display: inline-block;
  height: 28px;
  width: 120px;
}

/* ============ Variant 10 — diff view ============ */

.diff {
  background: var(--card);
  border: 1px solid var(--line);
  border-radius: var(--radius-card);
  font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
  margin: 0 auto;
  max-width: var(--max-width);
  overflow: hidden;
  width: 100%;
}

.diff-bar {
  align-items: center;
  background: var(--bg-tint);
  border-bottom: 1px solid var(--line);
  display: flex;
  font-size: 12px;
  gap: 12px;
  padding: 14px 22px;
}

.diff-file {
  border: 1px solid var(--line);
  border-radius: 4px;
  font-weight: 500;
  padding: 4px 10px;
}

.diff-file-old {
  background: rgb(220 38 38 / 8%);
  color: #b91c1c;
}

.diff-file-new {
  background: rgb(22 163 74 / 10%);
  color: #15803d;
}

.diff-arrow {
  color: var(--ink-mute);
  font-size: 14px;
}

.diff-stat-pill {
  background: var(--ink);
  border-radius: 4px;
  color: var(--bg);
  font-size: 11px;
  font-weight: 600;
  letter-spacing: 0.1em;
  margin-left: auto;
  padding: 4px 10px;
}

.diff-body {
  background: var(--code-bg);
  font-size: 13px;
  line-height: 1.75;
  margin: 0;
  padding: 14px 0;
}

.dl {
  display: block;
  padding: 0 22px;
  position: relative;
}

.dl.hunk {
  background: rgb(56 189 248 / 8%);
  color: #0369a1;
  font-weight: 600;
  letter-spacing: 0.04em;
  padding: 4px 22px;
}

.dl.ctx {
  color: var(--ink-soft);
}

.dl.add {
  background: rgb(22 163 74 / 12%);
  color: #15803d;
}

.dl.del {
  background: rgb(220 38 38 / 10%);
  color: #b91c1c;
}

.diff-foot {
  align-items: center;
  background: var(--bg-tint);
  border-top: 1px solid var(--line);
  display: flex;
  font-size: 12px;
  gap: 14px;
  padding: 14px 22px;
}

.diff-foot-stat {
  font-weight: 600;
  letter-spacing: 0.06em;
}

.diff-foot-stat.add { color: #16a34a; }
.diff-foot-stat.del { color: #dc2626; }
.diff-foot-stat.net {
  color: var(--ink);
  margin-right: auto;
}

.diff-foot-action {
  background: var(--ink);
  border-radius: 6px;
  color: var(--bg);
  font-weight: 600;
  letter-spacing: 0.04em;
  padding: 6px 14px;
}

/* ============ Receipt variant — A · thermal slim ============ */

.rcpt-thermal-wrap {
  display: grid;
  margin: 0 auto;
  max-width: var(--max-width);
  place-items: center;
  width: 100%;
}

.rcpt-thermal {
  background: var(--bg);
  border: 1px solid var(--line);
  color: var(--ink);
  display: grid;
  font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
  gap: 10px;
  max-width: 380px;
  padding: 30px 26px 26px;
  position: relative;
  width: 100%;
}

.rcpt-thermal::before,
.rcpt-thermal::after,
.rcpt-tape::before,
.rcpt-tape::after,
.rcpt-tape-tear {
  background-image:
    linear-gradient(135deg, var(--bg) 25%, transparent 25.5%),
    linear-gradient(45deg, var(--bg) 25%, transparent 25.5%);
  background-position: top left, top left;
  background-repeat: repeat-x;
  background-size: 14px 10px;
  content: "";
  height: 10px;
  left: 0;
  position: absolute;
  right: 0;
}

.rcpt-thermal::before { top: -1px; transform: rotate(180deg); }
.rcpt-thermal::after { bottom: -1px; }

.rcpt-th-head {
  text-align: center;
}

.rcpt-th-stamp {
  background: var(--ink);
  color: var(--bg);
  display: inline-block;
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 0.28em;
  margin-bottom: 6px;
  padding: 5px 12px;
}

.rcpt-th-sub {
  color: var(--ink-mute);
  display: block;
  font-size: 10px;
  letter-spacing: 0.2em;
  text-transform: uppercase;
}

.rcpt-th-meta {
  color: var(--ink-mute);
  display: flex;
  font-size: 11px;
  justify-content: space-between;
  letter-spacing: 0.16em;
  text-transform: uppercase;
}

.rcpt-th-meta span:last-child {
  color: var(--ink);
}

.rcpt-th-rule {
  border-top: 1px dashed var(--ink-mute);
  margin: 4px 0;
}

.rcpt-th-rule.double {
  border-top: 1px solid var(--ink);
  border-bottom: 1px solid var(--ink);
  height: 4px;
  margin: 8px 0;
}

.rcpt-th-items {
  display: grid;
  gap: 6px;
  list-style: none;
  margin: 0;
  padding: 0;
}

.rcpt-th-items li {
  align-items: baseline;
  display: grid;
  font-size: 12px;
  gap: 6px;
  grid-template-columns: minmax(0, auto) minmax(20px, 1fr) auto;
  letter-spacing: 0.04em;
}

.rcpt-th-items li.rcpt-th-opt span:first-child,
.rcpt-th-items li.rcpt-th-opt .rcpt-th-val {
  color: var(--ink-mute);
}

.rcpt-th-dots {
  border-bottom: 1px dotted var(--ink-mute);
  height: 0;
  margin-bottom: 4px;
}

.rcpt-th-val {
  color: var(--ink);
  font-weight: 700;
  letter-spacing: 0.1em;
  text-transform: uppercase;
  white-space: nowrap;
}

.rcpt-th-total {
  align-items: baseline;
  display: flex;
  font-size: 12px;
  font-weight: 700;
  justify-content: space-between;
  letter-spacing: 0.18em;
  text-transform: uppercase;
}

.rcpt-th-zero {
  color: var(--accent);
  font-size: clamp(28px, 3vw, 38px);
  font-weight: 700;
  letter-spacing: -0.02em;
}

.rcpt-th-foot {
  color: var(--ink-mute);
  font-size: 10px;
  letter-spacing: 0.2em;
  margin-top: 4px;
  text-align: center;
  text-transform: uppercase;
}

.rcpt-th-tear {
  display: none;
}

/* ============ Receipt variant — B · three-pass tape ============ */

.rcpt-tape-wrap {
  display: grid;
  margin: 0 auto;
  max-width: var(--max-width);
  place-items: center;
  width: 100%;
}

.rcpt-tape {
  background: var(--bg);
  border: 1px solid var(--line);
  color: var(--ink);
  display: grid;
  font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
  gap: 16px;
  max-width: 420px;
  padding: 30px 26px 24px;
  position: relative;
  width: 100%;
}

.rcpt-tape-tear.top { top: -1px; transform: rotate(180deg); }
.rcpt-tape-tear.bottom { bottom: -1px; }

.rcpt-tape-head {
  text-align: center;
}

.rcpt-tape-stamp {
  background: var(--ink);
  color: var(--bg);
  display: inline-block;
  font-size: 12px;
  font-weight: 700;
  letter-spacing: 0.28em;
  margin-bottom: 8px;
  padding: 5px 14px;
}

.rcpt-tape-no {
  color: var(--ink-mute);
  display: block;
  font-size: 10px;
  letter-spacing: 0.18em;
  text-transform: uppercase;
}

.rcpt-tape-pass {
  display: grid;
  gap: 4px;
}

.rcpt-tape-passhead {
  color: var(--ink);
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 0.18em;
  margin: 0 0 6px;
  text-align: center;
}

.rcpt-tape-rows {
  display: grid;
  gap: 4px;
  list-style: none;
  margin: 0;
  padding: 0;
}

.rcpt-tape-rows li {
  align-items: baseline;
  border-bottom: 1px dotted var(--ink-mute);
  display: flex;
  font-size: 12px;
  gap: 12px;
  justify-content: space-between;
  padding: 4px 0;
}

.rcpt-tape-val {
  color: var(--ink);
  font-weight: 600;
  letter-spacing: 0.06em;
  text-transform: uppercase;
  white-space: nowrap;
}

.rcpt-tape-skip {
  color: var(--ink-mute);
  text-decoration: line-through;
}

.rcpt-tape-sub {
  align-items: baseline;
  color: var(--ink);
  display: flex;
  font-size: 11px;
  font-weight: 600;
  justify-content: space-between;
  letter-spacing: 0.16em;
  margin-top: 6px;
  padding-top: 6px;
  text-transform: uppercase;
  border-top: 1px dashed var(--ink);
}

.rcpt-tape-rule {
  border-top: 1px dashed var(--ink);
}

.rcpt-tape-rule.double {
  border-top: 1px solid var(--ink);
  border-bottom: 1px solid var(--ink);
  height: 4px;
}

.rcpt-tape-total {
  align-items: baseline;
  display: flex;
  font-size: 13px;
  font-weight: 700;
  justify-content: space-between;
  letter-spacing: 0.18em;
  text-transform: uppercase;
}

.rcpt-tape-zero {
  color: var(--accent);
  font-size: clamp(32px, 3.4vw, 44px);
  font-weight: 700;
  letter-spacing: -0.03em;
}

.rcpt-tape-foot {
  color: var(--ink-mute);
  display: flex;
  font-size: 11px;
  justify-content: space-between;
  letter-spacing: 0.06em;
  margin-top: 4px;
}

/* ============ Receipt variant — C · carbon triplicate ============ */

.rcpt-carbon-wrap {
  align-items: center;
  display: grid;
  gap: 32px;
  grid-template-columns: minmax(0, 1fr) minmax(0, 1fr);
  margin: 0 auto;
  max-width: var(--max-width);
  width: 100%;
}

.rcpt-carbon-stack {
  position: relative;
  height: 480px;
  display: grid;
  place-items: center;
}

.rcpt-carbon {
  background: var(--bg);
  border: 1px solid var(--line);
  box-shadow: 0 18px 36px -28px rgb(0 0 0 / 30%);
  font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
  height: 380px;
  left: 50%;
  padding: 22px 24px;
  position: absolute;
  top: 50%;
  width: 320px;
}

.rcpt-carbon.copy-3 {
  background: var(--light, #fff8d8) var(--dark, #2c2818);
  transform: translate(calc(-50% + 28px), calc(-50% + 22px)) rotate(3.5deg);
  z-index: 1;
}

.rcpt-carbon.copy-2 {
  background: var(--light, #ffe2e6) var(--dark, #2a1d22);
  transform: translate(calc(-50% + 14px), calc(-50% + 10px)) rotate(-2deg);
  z-index: 2;
}

.rcpt-carbon.copy-1 {
  background: var(--bg);
  transform: translate(-50%, -50%) rotate(-0.6deg);
  z-index: 3;
}

.rcpt-cb-head {
  align-items: center;
  border-bottom: 1px dashed var(--ink-mute);
  display: flex;
  justify-content: space-between;
  padding-bottom: 8px;
}

.rcpt-cb-color-tag {
  background: var(--ink);
  color: var(--bg);
  font-size: 9px;
  font-weight: 700;
  letter-spacing: 0.22em;
  padding: 3px 7px;
}

.rcpt-cb-stamp {
  color: var(--ink-mute);
  font-size: 10px;
  font-weight: 700;
  letter-spacing: 0.24em;
}

.rcpt-cb-body {
  display: grid;
  gap: 10px;
  padding-top: 12px;
  position: relative;
}

.rcpt-cb-meta {
  color: var(--ink-mute);
  display: flex;
  font-size: 10px;
  justify-content: space-between;
  letter-spacing: 0.18em;
  text-transform: uppercase;
}

.rcpt-cb-meta span:first-child {
  color: var(--ink);
  font-weight: 600;
}

.rcpt-cb-rows {
  display: grid;
  gap: 5px;
  list-style: none;
  margin: 0;
  padding: 0;
}

.rcpt-cb-rows li {
  align-items: baseline;
  border-bottom: 1px dotted var(--ink-mute);
  display: flex;
  font-size: 11px;
  justify-content: space-between;
  padding: 3px 0;
  letter-spacing: 0.04em;
}

.rcpt-cb-rows li.rcpt-cb-opt {
  color: var(--ink-mute);
}

.rcpt-cb-val {
  color: var(--ink);
  font-weight: 600;
  letter-spacing: 0.1em;
  text-transform: uppercase;
}

.rcpt-cb-zero {
  color: var(--accent);
  font-weight: 700;
  letter-spacing: -0.02em;
}

.rcpt-cb-rule {
  border-top: 1px solid var(--ink);
  margin: 6px 0 4px;
}

.rcpt-cb-total {
  align-items: baseline;
  display: flex;
  font-size: 11px;
  font-weight: 700;
  justify-content: space-between;
  letter-spacing: 0.16em;
  text-transform: uppercase;
}

.rcpt-cb-total .rcpt-cb-zero {
  font-size: clamp(22px, 2.4vw, 30px);
}

.rcpt-cb-stamp-overlay {
  border: 3px double #dc2626;
  color: #dc2626;
  font-size: 22px;
  font-weight: 700;
  left: 60%;
  letter-spacing: 0.18em;
  opacity: 0.55;
  padding: 6px 16px;
  position: absolute;
  top: 30%;
  transform: rotate(-12deg);
}

.rcpt-carbon-legend {
  display: grid;
  gap: 14px;
  font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
  font-size: 13px;
  color: var(--ink-soft);
}

.rcpt-carbon-legend > div {
  align-items: center;
  display: flex;
  gap: 12px;
}

.rcpt-carbon-key {
  border: 1px solid var(--line);
  display: inline-block;
  height: 22px;
  width: 22px;
}

.rcpt-carbon-key.copy-1 { background: var(--bg); }
.rcpt-carbon-key.copy-2 { background: var(--light, #ffe2e6) var(--dark, #2a1d22); }
.rcpt-carbon-key.copy-3 { background: var(--light, #fff8d8) var(--dark, #2c2818); }

/* ============ Receipt variant — D · paid invoice ============ */

.rcpt-inv {
  background: var(--bg);
  border: 1px solid var(--line);
  border-radius: 4px;
  box-shadow: var(--light, 0 32px 70px -40px rgb(0 0 0 / 35%)) var(--dark, 0 1px 0 0 rgb(255 255 255 / 4%) inset);
  font-family: Inter, sans-serif;
  margin: 0 auto;
  max-width: var(--max-width);
  overflow: hidden;
  position: relative;
  width: 100%;
}

.rcpt-inv-letterhead {
  align-items: end;
  border-bottom: 2px solid var(--ink);
  display: flex;
  gap: 32px;
  justify-content: space-between;
  padding: 28px 32px 22px;
}

.rcpt-inv-brand {
  display: grid;
  gap: 4px;
}

.rcpt-inv-mono {
  color: var(--ink);
  font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
  font-size: clamp(28px, 3.2vw, 38px);
  font-weight: 600;
  letter-spacing: -0.02em;
  line-height: 1;
}

.rcpt-inv-brand-sub {
  color: var(--ink-mute);
  font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
  font-size: 11px;
  letter-spacing: 0.18em;
  text-transform: uppercase;
}

.rcpt-inv-meta {
  display: grid;
  gap: 6px;
  font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
  font-size: 12px;
}

.rcpt-inv-meta > div {
  align-items: baseline;
  display: flex;
  gap: 18px;
  justify-content: space-between;
}

.rcpt-inv-meta-key {
  color: var(--ink-mute);
  letter-spacing: 0.16em;
  text-transform: uppercase;
}

.rcpt-inv-meta-val {
  color: var(--ink);
  font-weight: 500;
}

.rcpt-inv-parties {
  border-bottom: 1px solid var(--line);
  display: grid;
  grid-template-columns: 1fr 1fr;
}

.rcpt-inv-party {
  border-right: 1px solid var(--line);
  display: grid;
  font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
  font-size: 12px;
  gap: 4px;
  padding: 18px 32px;
}

.rcpt-inv-party:last-child {
  border-right: 0;
}

.rcpt-inv-party-key {
  color: var(--ink-mute);
  font-size: 10px;
  letter-spacing: 0.22em;
  margin-bottom: 4px;
  text-transform: uppercase;
}

.rcpt-inv-party strong {
  color: var(--ink);
  font-size: 14px;
  font-weight: 600;
}

.rcpt-inv-party span {
  color: var(--ink-soft);
}

.rcpt-inv-table {
  border-collapse: collapse;
  font-size: 13px;
  width: 100%;
}

.rcpt-inv-table th,
.rcpt-inv-table td {
  border-bottom: 1px solid var(--line);
  padding: 14px 16px;
  text-align: left;
  vertical-align: top;
}

.rcpt-inv-table th {
  background: var(--bg-tint);
  color: var(--ink-mute);
  font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
  font-size: 10px;
  font-weight: 600;
  letter-spacing: 0.22em;
  text-transform: uppercase;
}

.rcpt-inv-table th:first-child,
.rcpt-inv-table td:first-child {
  padding-left: 32px;
}

.rcpt-inv-table th:last-child,
.rcpt-inv-table td:last-child {
  padding-right: 32px;
  text-align: right;
}

.rcpt-inv-table .col-qty,
.rcpt-inv-table .col-unit,
.rcpt-inv-table .col-amt {
  width: 130px;
}

.rcpt-inv-table .col-qty,
.rcpt-inv-table .col-unit {
  text-align: right;
}

.rcpt-inv-table tbody td:nth-child(2),
.rcpt-inv-table tbody td:nth-child(3) {
  font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
  text-align: right;
}

.rcpt-inv-desc {
  color: var(--ink-mute);
  font-family: Inter, sans-serif;
  font-size: 12px;
}

.rcpt-inv-zero {
  color: #16a34a;
  font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
  font-weight: 700;
}

.rcpt-inv-optional td {
  color: var(--ink-mute);
}

.rcpt-inv-optional strong {
  color: var(--ink-mute);
}

.rcpt-inv-summary {
  align-items: center;
  background: var(--bg-tint);
  border-top: 2px solid var(--ink);
  display: grid;
  gap: 24px;
  grid-template-columns: minmax(0, 1fr) auto;
  padding: 22px 32px;
}

.rcpt-inv-summary-rows {
  display: grid;
  font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
  gap: 6px;
  margin-left: auto;
  min-width: 280px;
}

.rcpt-inv-row {
  align-items: baseline;
  color: var(--ink-soft);
  display: flex;
  font-size: 13px;
  gap: 24px;
  justify-content: space-between;
}

.rcpt-inv-row.total {
  border-top: 1px solid var(--ink);
  color: var(--ink);
  font-size: 18px;
  font-weight: 700;
  letter-spacing: 0.04em;
  padding-top: 10px;
}

.rcpt-inv-total-zero {
  color: var(--accent);
  font-size: clamp(24px, 2.6vw, 32px);
  letter-spacing: -0.02em;
}

.rcpt-inv-stamp {
  border: 4px double #dc2626;
  color: #dc2626;
  font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
  font-size: 22px;
  font-weight: 700;
  letter-spacing: 0.16em;
  opacity: 0.85;
  padding: 10px 22px;
  transform: rotate(-6deg);
}

.rcpt-inv-foot {
  background: var(--ink);
  color: var(--bg);
  font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
  font-size: 12px;
  letter-spacing: 0.04em;
  padding: 14px 32px;
}

.rcpt-inv-foot code {
  background: rgb(255 255 255 / 12%);
  border-radius: 3px;
  padding: 1px 6px;
}

/* ============ Receipt — E · diner check ============ */

.rcpt-diner {
  background: var(--bg);
  border: 1px solid var(--line);
  border-radius: 4px;
  box-shadow: var(--light, 0 24px 50px -32px rgb(0 0 0 / 30%)) var(--dark, 0 1px 0 0 rgb(255 255 255 / 4%) inset);
  font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
  margin: 0 auto;
  max-width: 580px;
  overflow: hidden;
  width: 100%;
}

.rcpt-diner-head {
  align-items: end;
  border-bottom: 2px dashed var(--ink-mute);
  display: flex;
  flex-wrap: wrap;
  gap: 18px;
  justify-content: space-between;
  padding: 22px 28px 18px;
}

.rcpt-diner-brand { display: grid; gap: 4px; }

.rcpt-diner-name {
  color: var(--ink);
  font-family: Georgia, "Times New Roman", serif;
  font-size: 22px;
  font-style: italic;
  font-weight: 700;
  letter-spacing: 0.02em;
}

.rcpt-diner-tag { color: var(--ink-mute); font-size: 10px; letter-spacing: 0.18em; text-transform: uppercase; }

.rcpt-diner-meta { display: grid; gap: 4px; }
.rcpt-diner-meta > div { align-items: baseline; display: flex; font-size: 11px; gap: 12px; justify-content: space-between; }
.rcpt-diner-meta-key { color: var(--ink-mute); letter-spacing: 0.16em; text-transform: uppercase; }
.rcpt-diner-meta-val { color: var(--ink); font-weight: 600; }

.rcpt-diner-items { display: grid; gap: 0; list-style: none; margin: 0; padding: 8px 0; }

.rcpt-diner-items li {
  align-items: baseline;
  border-bottom: 1px dotted var(--ink-mute);
  display: grid;
  font-size: 13px;
  gap: 14px;
  grid-template-columns: 56px minmax(0, 1fr) 100px;
  padding: 12px 28px;
}

.rcpt-diner-qty { color: var(--ink); font-weight: 700; letter-spacing: 0.06em; }
.rcpt-diner-desc { display: grid; gap: 2px; }
.rcpt-diner-desc strong { color: var(--ink); font-weight: 600; }
.rcpt-diner-desc span { color: var(--ink-mute); font-size: 11px; letter-spacing: 0.04em; }
.rcpt-diner-price { color: var(--ink); font-weight: 700; letter-spacing: 0.06em; text-align: right; }

.rcpt-diner-side .rcpt-diner-qty,
.rcpt-diner-side .rcpt-diner-desc strong { color: var(--ink-mute); }

.rcpt-diner-tally {
  border-top: 2px solid var(--ink);
  display: grid;
  gap: 4px;
  padding: 14px 28px 12px;
}

.rcpt-diner-row { align-items: baseline; color: var(--ink-soft); display: flex; font-size: 13px; justify-content: space-between; }

.rcpt-diner-row.total {
  border-top: 1px dashed var(--ink);
  color: var(--ink);
  font-size: 16px;
  font-weight: 700;
  letter-spacing: 0.12em;
  margin-top: 6px;
  padding-top: 8px;
  text-transform: uppercase;
}

.rcpt-diner-zero { color: var(--accent); font-size: clamp(28px, 3vw, 38px); letter-spacing: -0.02em; }

.rcpt-diner-foot {
  align-items: end;
  background: var(--bg-tint);
  border-top: 1px solid var(--line);
  display: flex;
  flex-wrap: wrap;
  font-size: 11px;
  gap: 18px;
  justify-content: space-between;
  letter-spacing: 0.14em;
  padding: 14px 28px;
  text-transform: uppercase;
}

.rcpt-diner-thanks {
  color: var(--ink);
  font-family: Georgia, "Times New Roman", serif;
  font-size: 14px;
  font-style: italic;
  letter-spacing: 0.02em;
  text-transform: none;
}

.rcpt-diner-tip { display: grid; gap: 2px; text-align: right; }
.rcpt-diner-tip-key { color: var(--ink-mute); }
.rcpt-diner-tip-val { color: var(--ink); font-weight: 600; }

/* ============ Receipt — F · ATM withdrawal ============ */

.rcpt-atm-wrap { display: grid; margin: 0 auto; max-width: var(--max-width); place-items: center; width: 100%; }

.rcpt-atm {
  background: var(--light, #f4f1e8) var(--dark, #1f1d18);
  border: 1px solid var(--line);
  color: var(--light, #1a1a1a) var(--dark, #d8d2bf);
  display: grid;
  font-family: "Courier New", ui-monospace, SFMono-Regular, monospace;
  gap: 10px;
  max-width: 380px;
  padding: 26px 24px 22px;
  position: relative;
  width: 100%;
}

.rcpt-atm::before,
.rcpt-atm::after {
  background: repeating-linear-gradient(90deg, transparent 0 6px, currentColor 6px 9px);
  content: "";
  height: 4px;
  left: 0;
  opacity: 0.5;
  position: absolute;
  right: 0;
}
.rcpt-atm::before { top: 0; }
.rcpt-atm::after { bottom: 0; }

.rcpt-atm-head {
  align-items: baseline;
  border-bottom: 1px solid currentColor;
  display: flex;
  justify-content: space-between;
  padding-bottom: 6px;
  text-transform: uppercase;
}

.rcpt-atm-bank { font-size: 14px; font-weight: 700; letter-spacing: 0.18em; }

.rcpt-atm-stamp {
  background: currentColor;
  font-size: 9px;
  font-weight: 700;
  letter-spacing: 0.22em;
  padding: 3px 6px;
}

.rcpt-atm-stamp::before {
  color: var(--light, #f4f1e8) var(--dark, #1f1d18);
  content: "TXN COMPLETE";
}

.rcpt-atm-stamp { color: transparent; }

.rcpt-atm-meta {
  display: flex;
  font-size: 11px;
  justify-content: space-between;
  letter-spacing: 0.1em;
  text-transform: uppercase;
}
.rcpt-atm-meta span:first-child { opacity: 0.6; }
.rcpt-atm-meta span:last-child { font-weight: 700; }

.rcpt-atm-rule { border-top: 1px dashed currentColor; margin: 6px 0; opacity: 0.6; }

.rcpt-atm-block { display: grid; gap: 4px; padding: 4px 0; }
.rcpt-atm-key { font-size: 10px; letter-spacing: 0.18em; opacity: 0.65; text-transform: uppercase; }
.rcpt-atm-val { font-size: 14px; font-weight: 700; letter-spacing: 0.04em; }
.rcpt-atm-val.big { font-size: clamp(28px, 3vw, 38px); letter-spacing: -0.02em; }
.rcpt-atm-sub { font-size: 10px; letter-spacing: 0.04em; opacity: 0.6; }

.rcpt-atm-balance { align-items: baseline; display: flex; font-size: 12px; justify-content: space-between; letter-spacing: 0.06em; }
.rcpt-atm-bal-key { opacity: 0.6; text-transform: uppercase; }
.rcpt-atm-bal-val { font-weight: 700; }
.rcpt-atm-bal-val.zero { color: #16a34a; }

.rcpt-atm-foot { border-top: 1px dashed currentColor; display: grid; gap: 8px; padding-top: 10px; }
.rcpt-atm-foot-line { font-size: 9px; letter-spacing: 0.12em; opacity: 0.6; text-align: center; text-transform: uppercase; }

.rcpt-atm-foot-mag {
  background: repeating-linear-gradient(90deg, currentColor 0 2px, transparent 2px 4px, currentColor 4px 7px, transparent 7px 10px);
  height: 18px;
  margin-top: 4px;
  opacity: 0.85;
}

/* ============ Receipt — G · boarding pass ============ */

.rcpt-bp {
  background: var(--card);
  border: 1px solid var(--line);
  border-radius: 8px;
  box-shadow: var(--shadow-card);
  display: grid;
  font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
  grid-template-columns: minmax(0, 2.5fr) minmax(0, 1fr);
  margin: 0 auto;
  max-width: var(--max-width);
  overflow: hidden;
  position: relative;
  width: 100%;
}

.rcpt-bp-main {
  border-right: 2px dashed var(--ink-mute);
  display: grid;
  gap: 18px;
  padding: 24px 28px 22px;
  position: relative;
}

.rcpt-bp-airline { align-items: center; border-bottom: 1px solid var(--line); display: flex; gap: 18px; padding-bottom: 14px; }

.rcpt-bp-mark {
  background: var(--ink);
  border-radius: 6px;
  color: var(--bg);
  display: inline-flex;
  font-size: 22px;
  height: 44px;
  justify-content: center;
  line-height: 1;
  padding-top: 10px;
  width: 44px;
}

.rcpt-bp-line { color: var(--ink); display: block; font-size: 16px; font-weight: 700; letter-spacing: 0.18em; }
.rcpt-bp-class { color: var(--ink-mute); display: block; font-size: 11px; letter-spacing: 0.2em; text-transform: uppercase; }

.rcpt-bp-class-tag {
  background: var(--accent);
  border-radius: 4px;
  color: #fff;
  font-size: 10px;
  font-weight: 700;
  letter-spacing: 0.18em;
  margin-left: auto;
  padding: 5px 10px;
  text-transform: uppercase;
}

.rcpt-bp-route { align-items: end; display: grid; gap: 16px; grid-template-columns: minmax(0, 1fr) auto minmax(0, 1fr); }

.rcpt-bp-port { display: grid; gap: 4px; }
.rcpt-bp-port-code { color: var(--ink); font-family: ui-monospace, SFMono-Regular, Menlo, monospace; font-size: clamp(36px, 4.4vw, 56px); font-weight: 700; letter-spacing: -0.02em; line-height: 1; }
.rcpt-bp-port-name { color: var(--ink-mute); font-size: 11px; letter-spacing: 0.12em; text-transform: uppercase; }

.rcpt-bp-route > div:last-child .rcpt-bp-port-code,
.rcpt-bp-route > div:last-child .rcpt-bp-port-name { text-align: right; }

.rcpt-bp-arrow { color: var(--ink-mute); font-size: 22px; letter-spacing: 0.2em; padding-bottom: 16px; }

.rcpt-bp-grid { display: grid; gap: 14px 22px; grid-template-columns: repeat(3, minmax(0, 1fr)); }
.rcpt-bp-grid > div { display: grid; gap: 4px; }

.rcpt-bp-key { color: var(--ink-mute); font-size: 10px; letter-spacing: 0.22em; text-transform: uppercase; }
.rcpt-bp-val { color: var(--ink); font-size: 16px; font-weight: 600; }
.rcpt-bp-val.zero { color: var(--accent); font-size: 22px; }

.rcpt-bp-warn {
  background: var(--bg-tint);
  border: 1px solid var(--line);
  border-left: 3px solid #f59e0b;
  border-radius: 4px;
  color: var(--ink-soft);
  font-size: 12px;
  letter-spacing: 0.04em;
  line-height: 1.5;
  padding: 10px 14px;
}

.rcpt-bp-warn-tag {
  background: #f59e0b;
  border-radius: 3px;
  color: #1f2937;
  font-size: 10px;
  font-weight: 700;
  letter-spacing: 0.2em;
  margin-right: 10px;
  padding: 3px 6px;
}

.rcpt-bp-stub { background: var(--bg-tint); display: grid; gap: 14px; padding: 24px 22px; position: relative; }

.rcpt-bp-stub::before,
.rcpt-bp-stub::after {
  background: var(--bg);
  border: 1px solid var(--line);
  border-radius: 50%;
  content: "";
  height: 22px;
  left: -12px;
  position: absolute;
  width: 22px;
}
.rcpt-bp-stub::before { top: -12px; }
.rcpt-bp-stub::after { bottom: -12px; }

.rcpt-bp-stub-block { display: grid; gap: 2px; }
.rcpt-bp-stub-key { color: var(--ink-mute); font-size: 9px; letter-spacing: 0.22em; text-transform: uppercase; }
.rcpt-bp-stub-val { color: var(--ink); font-size: 14px; font-weight: 600; }

.rcpt-bp-stub-zero { background: var(--ink); border-radius: 4px; display: grid; gap: 4px; padding: 10px 12px; }
.rcpt-bp-stub-zero .rcpt-bp-stub-key { color: rgb(255 255 255 / 60%); }
.rcpt-bp-stub-zero-val { color: var(--accent); font-size: 26px; font-weight: 700; letter-spacing: -0.02em; }

.rcpt-bp-barcode {
  background: repeating-linear-gradient(90deg, var(--ink) 0 2px, transparent 2px 4px, var(--ink) 4px 5px, transparent 5px 8px, var(--ink) 8px 11px, transparent 11px 13px);
  height: 36px;
  margin-top: auto;
}

/* ============ Receipt — H · voided runtime ============ */

.rcpt-void {
  background: var(--bg);
  border: 1px solid var(--line);
  border-radius: 4px;
  box-shadow: var(--light, 0 28px 60px -36px rgb(0 0 0 / 32%)) var(--dark, 0 1px 0 0 rgb(255 255 255 / 4%) inset);
  display: grid;
  font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
  gap: 16px;
  margin: 0 auto;
  max-width: 560px;
  padding: 28px 30px 26px;
  position: relative;
  width: 100%;
  overflow: hidden;
}

.rcpt-void-head {
  align-items: center;
  border-bottom: 2px solid var(--ink);
  display: flex;
  flex-wrap: wrap;
  gap: 18px;
  justify-content: space-between;
  padding-bottom: 14px;
  position: relative;
  z-index: 1;
}

.rcpt-void-stamp-mark {
  background: #fee2e2;
  border: 2px solid #dc2626;
  color: #dc2626;
  font-size: 16px;
  font-weight: 700;
  letter-spacing: 0.32em;
  padding: 6px 14px;
  transform: rotate(-3deg);
}

.rcpt-void-meta { align-items: baseline; display: flex; font-size: 11px; gap: 10px; letter-spacing: 0.14em; text-transform: uppercase; }
.rcpt-void-meta-key { color: var(--ink-mute); }
.rcpt-void-meta-val { color: var(--ink); font-weight: 600; }

.rcpt-void-items { display: grid; gap: 4px; list-style: none; margin: 0; padding: 0; position: relative; z-index: 1; }

.rcpt-void-row {
  align-items: baseline;
  display: grid;
  font-size: 13px;
  gap: 12px;
  grid-template-columns: 24px minmax(0, 1fr) auto;
  padding: 8px 0;
  position: relative;
}

.rcpt-void-row.keep { border-bottom: 1px dotted var(--ink-mute); }

.rcpt-void-mark { color: var(--accent); font-weight: 700; text-align: center; }
.rcpt-void-mark.void { color: #dc2626; }

.rcpt-void-label { color: var(--ink); }
.rcpt-void-val { color: var(--ink); font-weight: 700; letter-spacing: 0.1em; text-transform: uppercase; }

.rcpt-void-row.voided {
  background: rgb(220 38 38 / 8%);
  color: var(--ink-mute);
  margin: 0 -12px;
  padding: 8px 12px;
  text-decoration: line-through;
}

.rcpt-void-row.voided .rcpt-void-label,
.rcpt-void-row.voided .rcpt-void-val { color: var(--ink-mute); }

.rcpt-void-strike {
  background: #dc2626;
  height: 1px;
  left: 12px;
  opacity: 0.5;
  position: absolute;
  right: 12px;
  top: 50%;
}

.rcpt-void-rule { border-top: 1px solid var(--ink); position: relative; z-index: 1; }

.rcpt-void-summary { display: grid; gap: 8px; position: relative; z-index: 1; }

.rcpt-void-summary > div { align-items: baseline; color: var(--ink-soft); display: flex; font-size: 12px; justify-content: space-between; }
.rcpt-void-summary-key { letter-spacing: 0.14em; text-transform: uppercase; }
.rcpt-void-summary-val { color: var(--ink); font-weight: 700; }

.rcpt-void-total {
  align-items: baseline;
  border-top: 1px dashed var(--ink);
  display: flex;
  font-size: 13px;
  font-weight: 700;
  justify-content: space-between;
  letter-spacing: 0.16em;
  margin-top: 6px;
  padding-top: 10px;
  text-transform: uppercase;
}

.rcpt-void-total-val { color: var(--accent); font-size: clamp(28px, 3vw, 40px); letter-spacing: -0.02em; }

.rcpt-void-foot {
  background: var(--bg-tint);
  color: var(--ink-mute);
  font-size: 11px;
  letter-spacing: 0.04em;
  margin: 8px -30px -26px;
  padding: 12px 30px;
  position: relative;
  z-index: 1;
}

.rcpt-void-foot code {
  background: var(--card);
  border: 1px solid var(--line);
  border-radius: 4px;
  padding: 1px 5px;
}

.rcpt-void-watermark {
  color: rgb(220 38 38 / 7%);
  font-size: clamp(120px, 18vw, 220px);
  font-weight: 900;
  letter-spacing: 0.05em;
  pointer-events: none;
  position: absolute;
  right: -18px;
  top: 30%;
  transform: rotate(-12deg);
  user-select: none;
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
  align-items: stretch;
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
  display: flex;
  flex-direction: column;
  min-height: 460px;
  overflow: hidden;
}

.demo-pane-frame .card-bar {
  background: var(--card);
  border-bottom: 1px solid var(--line);
}

.demo-pane-frame .card-code-body {
  display: flex;
  flex: 1 1 auto;
  min-height: 0;
}

.demo-pane-frame .card-code-body .dxc {
  display: flex;
  flex: 1 1 auto;
  flex-direction: column;
  justify-content: center;
  font-size: 14px;
  line-height: 1.65;
  padding: 22px 26px;
}

/* ============ Docs ============ */

.docs-timeline {
  display: grid;
  gap: 0;
  list-style: none;
  margin: 0 auto;
  max-width: var(--max-width);
  padding: 0;
  width: 100%;
}

.docs-timeline-step {
  display: grid;
  gap: 28px;
  grid-template-columns: 88px minmax(0, 1fr);
  padding: 36px 6px;
  position: relative;
}

.docs-timeline-step + .docs-timeline-step {
  border-top: 1px solid var(--line);
}

.docs-timeline-rail {
  display: flex;
  justify-content: flex-end;
  padding-top: 8px;
  position: relative;
}

.docs-timeline-rail::after {
  background: var(--line);
  bottom: -36px;
  content: "";
  left: calc(100% - 22px);
  position: absolute;
  top: 56px;
  transform: translateX(-50%);
  width: 1px;
}

.docs-timeline-step:last-child .docs-timeline-rail::after {
  display: none;
}

.docs-timeline-num {
  background: var(--bg-tint);
  border: 1px solid var(--line);
  border-radius: 999px;
  color: var(--ink);
  display: inline-flex;
  align-items: center;
  justify-content: center;
  font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
  font-size: 13px;
  font-weight: 600;
  height: 44px;
  letter-spacing: 0.04em;
  width: 44px;
}

.docs-timeline-content {
  display: grid;
  gap: 14px;
  min-width: 0;
}

.docs-timeline-title {
  color: var(--ink);
  font-family: Inter, sans-serif;
  font-size: clamp(24px, 2.4vw, 34px);
  font-weight: 600;
  letter-spacing: -0.02em;
  line-height: 1.15;
  margin: 0;
  max-width: 24ch;
}

.docs-timeline-copy {
  color: var(--ink-soft);
  font-family: Inter, sans-serif;
  font-size: 15px;
  line-height: 1.6;
  margin: 0;
  max-width: 60ch;
}

.docs-timeline-frame {
  background: var(--code-bg);
  border: 1px solid var(--line);
  border-radius: var(--radius-inner);
  margin-top: 4px;
  overflow: hidden;
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

  .feature-grid {
    grid-template-columns: 1fr;
  }

  .ledger-grid {
    grid-template-columns: 1fr;
  }

  .ledger-divider {
    height: 1px;
  }

  .ledger-foot {
    grid-template-columns: 1fr;
    gap: 8px;
  }

  .receipt-stack {
    grid-template-columns: 1fr;
  }

  .receipt {
    margin: 0 auto;
    width: 100%;
  }

  .codeblock-body {
    font-size: 12px;
  }

  .cb-line {
    grid-template-columns: 44px minmax(0, 1fr);
  }

  .specimen-stage {
    grid-template-columns: 1fr;
    grid-template-areas: "tl" "tr" "hero" "bl" "br";
    text-align: left;
    padding: 36px 24px;
  }

  .specimen-anno.tr,
  .specimen-anno.br {
    justify-self: start;
  }

  .specimen-hero {
    font-size: clamp(120px, 38vw, 220px);
  }

  .specimen-hero::before,
  .specimen-hero::after {
    display: none;
  }

  .specimen-strip {
    grid-template-columns: 1fr;
  }

  .specimen-sample {
    border-right: 0;
    border-bottom: 1px solid var(--line);
  }

  .specimen-sample:last-child {
    border-bottom: 0;
  }

  .punchcard-row {
    grid-template-columns: 110px minmax(0, 1fr);
  }

  .punchcard-meta {
    grid-column: 2 / -1;
    text-align: left;
  }

  .manifest-meta {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }

  .manifest-meta-cell:nth-child(2) {
    border-right: 0;
  }

  .manifest-meta-cell:nth-child(1),
  .manifest-meta-cell:nth-child(2) {
    border-bottom: 1px solid var(--line);
  }

  .rx-grid {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }

  .rx-field:nth-child(2) {
    border-right: 0;
  }

  .rx-field:nth-child(1),
  .rx-field:nth-child(2) {
    border-bottom: 1px dashed var(--line-strong);
  }

  .rcpt-carbon-wrap {
    grid-template-columns: 1fr;
    gap: 24px;
  }

  .rcpt-inv-letterhead {
    align-items: start;
    flex-direction: column;
    gap: 18px;
  }

  .rcpt-inv-summary {
    grid-template-columns: 1fr;
    text-align: center;
  }

  .rcpt-inv-summary-rows {
    margin: 0 auto;
  }

  .demo-pane {
    grid-template-columns: 1fr;
  }

  .docs-timeline-step {
    grid-template-columns: 64px minmax(0, 1fr);
    gap: 18px;
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

  .hero {
    padding: 24px 14px 40px;
  }

  .ledger-side {
    padding: 22px 18px;
  }

  .ledger-bar,
  .codeblock-bar,
  .codeblock-foot {
    padding: 12px 16px;
  }

  .receipt {
    padding: 28px 22px 26px;
  }

  .receipt-aside-row {
    grid-template-columns: 40px minmax(0, 1fr);
    gap: 12px;
  }

  .codeblock-body {
    padding: 16px 0;
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
