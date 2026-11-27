use dioxus::prelude::*;

/// A native `<select>`; pass `option` elements as children.
#[component]
pub fn Select(
    onchange: EventHandler<FormEvent>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    rsx! {
        document::Link { rel: "stylesheet", href: asset!("./style.css") }
        div { class: "select",
            select {
                class: "select-trigger",
                onchange: move |event| onchange.call(event),
                ..attributes,
                {children}
            }
            svg {
                class: "select-expand-icon",
                view_box: "0 0 24 24",
                xmlns: "http://www.w3.org/2000/svg",
                polyline { points: "6 9 12 15 18 9" }
            }
        }
    }
}
