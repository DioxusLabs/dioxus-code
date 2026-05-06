use dioxus::prelude::*;
use dioxus_code::{Code, Theme, code};

fn main() {
    dioxus::launch(TokenPalette);
}

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
