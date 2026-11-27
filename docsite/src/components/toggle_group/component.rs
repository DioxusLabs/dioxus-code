use dioxus::prelude::*;

#[component]
pub fn ToggleGroup(
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    rsx! {
        document::Link { rel: "stylesheet", href: asset!("./style.css") }
        div { class: "toggle-group", role: "group", ..attributes, {children} }
    }
}

#[component]
pub fn ToggleItem(
    pressed: bool,
    onclick: EventHandler<MouseEvent>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    rsx! {
        button {
            class: "toggle-item",
            r#type: "button",
            "aria-pressed": pressed,
            "data-state": if pressed { "on" } else { "off" },
            onclick: move |event| onclick.call(event),
            ..attributes,
            {children}
        }
    }
}
