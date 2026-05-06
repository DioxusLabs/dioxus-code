use std::collections::BTreeMap;
use std::env;
use std::fmt::Write as _;
use std::fs;
use std::path::PathBuf;

use arborium::theme::builtin;

struct ThemeDef {
    const_name: &'static str,
    theme: arborium::theme::Theme,
}

struct ParsedTheme {
    const_name: &'static str,
    name: String,
    css_file: String,
    system_light_css_file: String,
    system_dark_css_file: String,
    class: String,
    system_light_class: String,
    system_dark_class: String,
    rules: ThemeRules,
}

#[derive(Default)]
struct ThemeRules {
    base: Vec<Declaration>,
    tags: BTreeMap<String, Vec<Declaration>>,
}

#[derive(Clone)]
struct Declaration {
    property: String,
    value: String,
}

#[derive(Default)]
struct SharedRules {
    base: Vec<String>,
    tags: BTreeMap<String, Vec<String>>,
}

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let asset_dir = manifest_dir.join("assets/generated/arborium-themes");
    fs::create_dir_all(&asset_dir).unwrap();

    let themes: Vec<_> = themes().into_iter().map(parse_theme).collect();
    let mut shared_rules = SharedRules::default();
    let mut generated = String::from(
        r#"impl Theme {
    /// Shared stylesheet for syntax token rules and CSS-only system theme selection.
    pub const THEME_CSS: Asset = asset!("/assets/generated/arborium-themes/dioxus-code-theme.css");
"#,
    );

    for theme in &themes {
        shared_rules.insert(&theme.rules);
        fs::write(asset_dir.join(&theme.css_file), fixed_theme_css(theme)).unwrap();
        fs::write(
            asset_dir.join(&theme.system_light_css_file),
            system_slot_css(&theme.system_light_class, "light", &theme.rules),
        )
        .unwrap();
        fs::write(
            asset_dir.join(&theme.system_dark_css_file),
            system_slot_css(&theme.system_dark_class, "dark", &theme.rules),
        )
        .unwrap();

        generated.push_str(&format!(
            "    /// Stylesheet asset for the `{name}` theme.\n    pub const {const_name}_CSS: Asset = asset!(\"/assets/generated/arborium-themes/{css_file}\");\n",
            const_name = theme.const_name,
            css_file = theme.css_file,
            name = theme.name,
        ));
        generated.push_str(&format!(
            "    /// Stylesheet asset for the `{name}` theme's system light variables.\n    pub const {const_name}_SYSTEM_LIGHT_CSS: Asset = asset!(\"/assets/generated/arborium-themes/{css_file}\");\n",
            const_name = theme.const_name,
            css_file = theme.system_light_css_file,
            name = theme.name,
        ));
        generated.push_str(&format!(
            "    /// Stylesheet asset for the `{name}` theme's system dark variables.\n    pub const {const_name}_SYSTEM_DARK_CSS: Asset = asset!(\"/assets/generated/arborium-themes/{css_file}\");\n",
            const_name = theme.const_name,
            css_file = theme.system_dark_css_file,
            name = theme.name,
        ));

        generated.push_str(&format!(
            "    /// The `{name}` syntax theme.\n    pub const {const_name}: Self = Self {{ name: \"{name}\", class: \"{class}\", system_light_class: \"{system_light_class}\", system_dark_class: \"{system_dark_class}\", asset: Self::{const_name}_CSS, system_light_asset: Self::{const_name}_SYSTEM_LIGHT_CSS, system_dark_asset: Self::{const_name}_SYSTEM_DARK_CSS }};\n",
            const_name = theme.const_name,
            name = theme.name,
            class = theme.class,
            system_light_class = theme.system_light_class,
            system_dark_class = theme.system_dark_class,
        ));
    }

    generated.push_str(
        r#"}
"#,
    );

    fs::write(
        asset_dir.join("dioxus-code-theme.css"),
        shared_theme_css(&shared_rules),
    )
    .unwrap();

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    fs::write(out_dir.join("theme_assets.rs"), generated).unwrap();
}

fn parse_theme(theme: ThemeDef) -> ParsedTheme {
    let name = slug(&theme.theme.name);
    let css_file = format!("{name}.css");
    let system_light_css_file = format!("{name}-system-light.css");
    let system_dark_css_file = format!("{name}-system-dark.css");
    let class = format!("dxc-{name}");
    let system_light_class = format!("dxc-system-light-{name}");
    let system_dark_class = format!("dxc-system-dark-{name}");
    let selector = format!(".{class}");
    let rules = parse_theme_rules(&theme.theme.to_css(&selector));

    ParsedTheme {
        const_name: theme.const_name,
        name,
        css_file,
        system_light_css_file,
        system_dark_css_file,
        class,
        system_light_class,
        system_dark_class,
        rules,
    }
}

fn parse_theme_rules(css: &str) -> ThemeRules {
    let mut rules = ThemeRules::default();

    for line in css.lines().skip(1) {
        let trimmed = line.trim();
        if trimmed == "}" || trimmed.is_empty() {
            continue;
        }

        if trimmed.starts_with("a-") {
            let (tag, body) = trimmed.split_once('{').unwrap();
            let body = body.trim().trim_end_matches('}').trim();
            rules
                .tags
                .entry(tag.trim().to_string())
                .or_default()
                .extend(parse_declarations(body));
        } else {
            rules.base.extend(parse_declarations(trimmed));
        }
    }

    rules
}

fn parse_declarations(input: &str) -> Vec<Declaration> {
    input
        .split(';')
        .filter_map(|declaration| {
            let declaration = declaration.trim();
            if declaration.is_empty() {
                return None;
            }

            let (property, value) = declaration.split_once(':')?;
            Some(Declaration {
                property: property.trim().to_string(),
                value: value.trim().to_string(),
            })
        })
        .collect()
}

fn fixed_theme_css(theme: &ParsedTheme) -> String {
    let mut css = String::new();

    writeln!(css, ".{} {{", theme.class).unwrap();
    write_slot_variables(&mut css, "light", &theme.rules);
    css.push_str("}\n");

    css
}

fn system_slot_css(selector: &str, slot: &str, rules: &ThemeRules) -> String {
    let mut css = String::new();

    writeln!(css, ".{selector} {{").unwrap();
    write_slot_variables(&mut css, slot, rules);
    css.push_str("}\n");

    css
}

fn write_slot_variables(css: &mut String, slot: &str, rules: &ThemeRules) {
    for declaration in &rules.base {
        let variable = base_variable_suffix(&declaration.property);
        writeln!(css, "  --dxc-{slot}-{variable}: {};", declaration.value).unwrap();
    }

    for (tag, declarations) in &rules.tags {
        for declaration in declarations {
            let variable = tag_variable_suffix(tag, &declaration.property);
            writeln!(css, "  --dxc-{slot}-{variable}: {};", declaration.value).unwrap();
        }
    }
}

impl SharedRules {
    fn insert(&mut self, rules: &ThemeRules) {
        for declaration in &rules.base {
            insert_unique(&mut self.base, &declaration.property);
        }

        for (tag, declarations) in &rules.tags {
            let properties = self.tags.entry(tag.clone()).or_default();
            for declaration in declarations {
                insert_unique(properties, &declaration.property);
            }
        }
    }
}

fn insert_unique(values: &mut Vec<String>, value: &str) {
    if !values.iter().any(|existing| existing == value) {
        values.push(value.to_string());
    }
}

fn shared_theme_css(rules: &SharedRules) -> String {
    let mut css = String::from(
        r#".dxc-system {
  --dxc-light-on: initial;
  --dxc-dark-on: ;
}

@media (prefers-color-scheme: dark) {
  .dxc-system {
    --dxc-light-on: ;
    --dxc-dark-on: initial;
  }
}

"#,
    );

    css.push_str(".dxc,\n.dxc-editor {\n");
    for property in &rules.base {
        let variable = base_variable_suffix(property);
        writeln!(css, "  {property}: {};", active_value(&variable)).unwrap();
    }
    css.push_str("}\n");

    for (tag, properties) in &rules.tags {
        writeln!(css, ".dxc .{tag},\n.dxc-editor .{tag} {{").unwrap();
        for property in properties {
            let variable = tag_variable_suffix(tag, property);
            writeln!(css, "  {property}: {};", active_value(&variable)).unwrap();
        }
        css.push_str("}\n");
    }

    css
}

fn active_value(variable: &str) -> String {
    format!(
        "var(--dxc-light-on, var(--dxc-light-{variable},)) var(--dxc-dark-on, var(--dxc-dark-{variable},))"
    )
}

fn base_variable_suffix(property: &str) -> String {
    if let Some(custom_property) = property.strip_prefix("--") {
        format!("var-{}", css_identifier(custom_property))
    } else {
        css_identifier(property)
    }
}

fn tag_variable_suffix(tag: &str, property: &str) -> String {
    format!("{}-{}", css_identifier(tag), base_variable_suffix(property))
}

fn css_identifier(input: &str) -> String {
    let mut output = String::new();

    for ch in input.chars() {
        match ch {
            'a'..='z' | '0'..='9' => output.push(ch),
            'A'..='Z' => output.push(ch.to_ascii_lowercase()),
            '-' | '_' if !output.ends_with('-') => output.push('-'),
            _ => {}
        }
    }

    output.trim_matches('-').to_string()
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
