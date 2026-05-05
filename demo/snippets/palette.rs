use dioxus::prelude::*;
use dioxus_code::{Code, Theme, code};

#[component]
pub fn TokenPalette() -> Element {
    rsx! {
        section { class: "palette-demo",
            Code {
                src: code!("/snippets/palette.rs"),
                theme: Theme::KANAGAWA_DRAGON,
            }
        }
    }
}
