use dioxus::prelude::*;

#[component]
pub fn Separator(#[props(extends = GlobalAttributes)] attributes: Vec<Attribute>) -> Element {
    rsx! {
        document::Link { rel: "stylesheet", href: asset!("./style.css") }
        div {
            class: "separator",
            role: "separator",
            "data-orientation": "horizontal",
            ..attributes,
        }
    }
}
