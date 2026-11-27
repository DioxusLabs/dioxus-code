use dioxus::prelude::*;

#[component]
pub fn Navbar(
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    rsx! {
        document::Link { rel: "stylesheet", href: asset!("./style.css") }
        nav { class: "navbar", ..attributes, {children} }
    }
}

/// A dropdown group, opened on hover or when focus is inside it.
#[component]
pub fn NavbarNav(children: Element) -> Element {
    rsx! {
        div { class: "navbar-nav", {children} }
    }
}

#[component]
pub fn NavbarTrigger(children: Element) -> Element {
    rsx! {
        button { class: "navbar-trigger", r#type: "button", "aria-haspopup": "true",
            {children}
            svg {
                class: "navbar-expand-icon",
                view_box: "0 0 24 24",
                xmlns: "http://www.w3.org/2000/svg",
                polyline { points: "6 9 12 15 18 9" }
            }
        }
    }
}

#[component]
pub fn NavbarContent(children: Element) -> Element {
    rsx! {
        div { class: "navbar-content", {children} }
    }
}

#[component]
pub fn NavbarItem(
    #[props(into)] to: String,
    #[props(default)] new_tab: bool,
    children: Element,
) -> Element {
    rsx! {
        a {
            class: "navbar-item",
            href: to,
            target: if new_tab { "_blank" },
            rel: if new_tab { "noopener noreferrer" },
            {children}
        }
    }
}
