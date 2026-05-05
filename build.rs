use std::env;
use std::fs;
use std::path::PathBuf;

use arborium::theme::builtin;

struct ThemeDef {
    const_name: &'static str,
    theme: arborium::theme::Theme,
}

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let asset_dir = manifest_dir.join("assets/generated/arborium-themes");
    if !asset_dir.exists() {
        fs::create_dir_all(&asset_dir).unwrap();
    }

    let themes = themes();
    let mut generated = String::from(
        r#"impl Theme {
"#,
    );

    for theme in themes {
        let name = slug(&theme.theme.name);
        let css_file = format!("{name}.css");
        let class = format!("dxc-{name}");
        let selector = format!(".{class}");
        let css_path = asset_dir.join(&css_file);
        if !css_path.exists() {
            let css = flatten_theme_css(&selector, &theme.theme.to_css(&selector));
            fs::write(css_path, css).unwrap();
        }

        generated.push_str(&format!(
            "    /// Stylesheet asset for the `{name}` theme.\n    pub const {const_name}_CSS: Asset = asset!(\"/assets/generated/arborium-themes/{css_file}\");\n",
            const_name = theme.const_name,
            css_file = css_file,
            name = name,
        ));

        generated.push_str(&format!(
            "    /// The `{name}` syntax theme.\n    pub const {const_name}: Self = Self {{ name: \"{name}\", class: \"{class}\", asset: Self::{const_name}_CSS }};\n",
            const_name = theme.const_name,
            name = name,
            class = class,
        ));
    }

    generated.push_str(
        r#"}
"#,
    );

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    fs::write(out_dir.join("theme_assets.rs"), generated).unwrap();
}

fn flatten_theme_css(selector: &str, css: &str) -> String {
    let mut base = Vec::new();
    let mut nested = Vec::new();

    for line in css.lines().skip(1) {
        let trimmed = line.trim();
        if trimmed == "}" || trimmed.is_empty() {
            continue;
        }

        if trimmed.starts_with("a-") {
            let (tag, rest) = trimmed.split_once(' ').unwrap();
            nested.push(format!("{selector} .{tag} {rest}"));
        } else {
            base.push(format!("  {trimmed}"));
        }
    }

    let mut flattened = String::new();
    flattened.push_str(selector);
    flattened.push_str(" {\n");
    for line in base {
        flattened.push_str(&line);
        flattened.push('\n');
    }
    flattened.push_str("}\n");

    for line in nested {
        flattened.push_str(&line);
        flattened.push('\n');
    }

    flattened
}

fn themes() -> Vec<ThemeDef> {
    vec![
        theme("ALABASTER", builtin::alabaster()),
        theme("AYU_DARK", builtin::ayu_dark()),
        theme("AYU_LIGHT", builtin::ayu_light()),
        theme("CATPPUCCIN_FRAPPE", builtin::catppuccin_frappe()),
        theme("CATPPUCCIN_LATTE", builtin::catppuccin_latte()),
        theme("CATPPUCCIN_MACCHIATO", builtin::catppuccin_macchiato()),
        theme("CATPPUCCIN_MOCHA", builtin::catppuccin_mocha()),
        theme("COBALT2", builtin::cobalt2()),
        theme("DAYFOX", builtin::dayfox()),
        theme("DESERT256", builtin::desert256()),
        theme("DRACULA", builtin::dracula()),
        theme("EF_MELISSA_DARK", builtin::ef_melissa_dark()),
        theme("GITHUB_DARK", builtin::github_dark()),
        theme("GITHUB_LIGHT", builtin::github_light()),
        theme("GRUVBOX_DARK", builtin::gruvbox_dark()),
        theme("GRUVBOX_LIGHT", builtin::gruvbox_light()),
        theme("KANAGAWA_DRAGON", builtin::kanagawa_dragon()),
        theme("LIGHT_OWL", builtin::light_owl()),
        theme("LUCIUS_LIGHT", builtin::lucius_light()),
        theme("MELANGE_DARK", builtin::melange_dark()),
        theme("MELANGE_LIGHT", builtin::melange_light()),
        theme("MONOKAI", builtin::monokai()),
        theme("NORD", builtin::nord()),
        theme("ONE_DARK", builtin::one_dark()),
        theme("ROSE_PINE_MOON", builtin::rose_pine_moon()),
        theme("RUSTDOC_AYU", builtin::rustdoc_ayu()),
        theme("RUSTDOC_DARK", builtin::rustdoc_dark()),
        theme("RUSTDOC_LIGHT", builtin::rustdoc_light()),
        theme("SOLARIZED_DARK", builtin::solarized_dark()),
        theme("SOLARIZED_LIGHT", builtin::solarized_light()),
        theme("TOKYO_NIGHT", builtin::tokyo_night()),
        theme("ZENBURN", builtin::zenburn()),
    ]
}

fn theme(const_name: &'static str, theme: arborium::theme::Theme) -> ThemeDef {
    ThemeDef { const_name, theme }
}

fn slug(name: &str) -> String {
    let mut slug = String::new();

    for ch in name.chars() {
        match ch {
            'a'..='z' | '0'..='9' => slug.push(ch),
            'A'..='Z' => slug.push(ch.to_ascii_lowercase()),
            ' ' | '_' | '-' if !slug.ends_with('-') => slug.push('-'),
            'é' | 'É' => slug.push('e'),
            _ => {}
        }
    }

    slug.trim_matches('-').to_string()
}
