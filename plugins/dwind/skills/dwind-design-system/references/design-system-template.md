# Design System Starter Template

Copy-pasteable starter files for a dwind design system crate. Customize the color mappings, spacing scale, and typography roles for your project.

## Cargo.toml

```toml
[package]
name = "my-design-system"
version = "0.1.0"
edition = "2021"

[dependencies]
dwind = { git = "https://github.com/nicksenger/dominator-css-bindgen", features = ["default_colors"] }
dwind-macros = { git = "https://github.com/nicksenger/dominator-css-bindgen" }
dominator = "0.5"
futures-signals = "0.3"
web-sys = { version = "0.3", features = ["HtmlElement", "CssStyleDeclaration", "Document", "Window", "Element"] }
wasm-bindgen = "0.2"

[build-dependencies]
dominator-css-bindgen = { git = "https://github.com/nicksenger/dominator-css-bindgen" }
```

## tokens.css

```css
/* ============================================================
   PRIMITIVE TOKENS — raw values, named after what they ARE
   ============================================================ */

/* Spacing (4px base unit) */
.ds-space-1  { --ds-space-xs:  4px; }
.ds-space-2  { --ds-space-sm:  8px; }
.ds-space-4  { --ds-space-md:  16px; }
.ds-space-6  { --ds-space-lg:  24px; }
.ds-space-8  { --ds-space-xl:  32px; }
.ds-space-12 { --ds-space-2xl: 48px; }

/* Type sizes */
.ds-text-xs   { --ds-text-label:   12px; }
.ds-text-sm   { --ds-text-caption:  14px; }
.ds-text-base { --ds-text-body:     16px; }
.ds-text-xl   { --ds-text-heading3: 20px; }
.ds-text-2xl  { --ds-text-heading2: 24px; }
.ds-text-4xl  { --ds-text-heading1: 36px; }

/* ============================================================
   SEMANTIC TOKENS — purpose-based, named after what they DO
   Dark mode is the default (:root)
   ============================================================ */

/* Backgrounds */
.ds-bg           { color: var(--ds-color-bg); }
.ds-bg-elevated  { color: var(--ds-color-bg-elevated); }
.ds-bg-muted     { color: var(--ds-color-bg-muted); }

/* Text */
.ds-text         { color: var(--ds-color-text); }
.ds-text-muted   { color: var(--ds-color-text-muted); }
.ds-text-inverted { color: var(--ds-color-text-inverted); }

/* Interactive */
.ds-primary       { color: var(--ds-color-primary); }
.ds-primary-hover { color: var(--ds-color-primary-hover); }
.ds-secondary     { color: var(--ds-color-secondary); }

/* Borders */
.ds-border       { border-color: var(--ds-color-border); }
.ds-border-muted { border-color: var(--ds-color-border-muted); }

/* Status */
.ds-success { color: var(--ds-color-success); }
.ds-warning { color: var(--ds-color-warning); }
.ds-error   { color: var(--ds-color-error); }
.ds-info    { color: var(--ds-color-info); }
```

## theme/palettes.rs

```rust
/// Color values for each theme. Reference dwind's color palette
/// (see dwind-styling references/color-palette.md for hex values).
pub struct Palette {
    pub bg: &'static str,
    pub bg_elevated: &'static str,
    pub bg_muted: &'static str,
    pub text: &'static str,
    pub text_muted: &'static str,
    pub text_inverted: &'static str,
    pub primary: &'static str,
    pub primary_hover: &'static str,
    pub secondary: &'static str,
    pub border: &'static str,
    pub border_muted: &'static str,
    pub success: &'static str,
    pub warning: &'static str,
    pub error: &'static str,
    pub info: &'static str,
}

pub const DARK: Palette = Palette {
    bg:             "#111827", // gray-900
    bg_elevated:    "#1f2937", // gray-800
    bg_muted:       "#374151", // gray-700
    text:           "#f9fafb", // gray-50
    text_muted:     "#9ca3af", // gray-400
    text_inverted:  "#111827", // gray-900
    primary:        "#3b82f6", // blue-500
    primary_hover:  "#2563eb", // blue-600
    secondary:      "#8b5cf6", // purple-500
    border:         "#374151", // gray-700
    border_muted:   "#1f2937", // gray-800
    success:        "#4ade80", // green-400
    warning:        "#fbbf24", // yellow-400
    error:          "#f87171", // red-400
    info:           "#38bdf8", // blue-400 (picton-blue)
};

pub const LIGHT: Palette = Palette {
    bg:             "#f9fafb", // gray-50
    bg_elevated:    "#ffffff", // white
    bg_muted:       "#f3f4f6", // gray-100
    text:           "#111827", // gray-900
    text_muted:     "#6b7280", // gray-500
    text_inverted:  "#f9fafb", // gray-50
    primary:        "#2563eb", // blue-600 (darker for contrast on light bg)
    primary_hover:  "#1d4ed8", // blue-700
    secondary:      "#7c3aed", // purple-600
    border:         "#e5e7eb", // gray-200
    border_muted:   "#f3f4f6", // gray-100
    success:        "#16a34a", // green-600
    warning:        "#d97706", // yellow-600
    error:          "#dc2626", // red-600
    info:           "#0284c7", // blue-600
};
```

## theme/mod.rs

```rust
pub mod palettes;

use palettes::{Palette, DARK, LIGHT};
use web_sys::wasm_bindgen::JsCast;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Theme {
    Dark,
    Light,
}

impl Theme {
    pub fn palette(&self) -> &'static Palette {
        match self {
            Theme::Dark => &DARK,
            Theme::Light => &LIGHT,
        }
    }
}

/// Apply a theme by setting CSS custom properties on :root.
/// Call this on app init and when the user switches themes.
pub fn apply_theme(theme: Theme) {
    let Some(root) = web_sys::window()
        .and_then(|w| w.document())
        .and_then(|d| d.document_element())
    else {
        return;
    };

    let el: &web_sys::HtmlElement = root.unchecked_ref();
    let style = el.style();
    let p = theme.palette();

    let vars = [
        ("--ds-color-bg", p.bg),
        ("--ds-color-bg-elevated", p.bg_elevated),
        ("--ds-color-bg-muted", p.bg_muted),
        ("--ds-color-text", p.text),
        ("--ds-color-text-muted", p.text_muted),
        ("--ds-color-text-inverted", p.text_inverted),
        ("--ds-color-primary", p.primary),
        ("--ds-color-primary-hover", p.primary_hover),
        ("--ds-color-secondary", p.secondary),
        ("--ds-color-border", p.border),
        ("--ds-color-border-muted", p.border_muted),
        ("--ds-color-success", p.success),
        ("--ds-color-warning", p.warning),
        ("--ds-color-error", p.error),
        ("--ds-color-info", p.info),
    ];

    for (prop, val) in vars {
        let _ = style.set_property(prop, val);
    }

    // Toggle .light class for is(.light) selectors in dwclass!
    let class_list = root.class_list();
    match theme {
        Theme::Dark => { let _ = class_list.remove_1("light"); },
        Theme::Light => { let _ = class_list.add_1("light"); },
    }
}
```

## mixins/typography.rs

```rust
use dominator::DomBuilder;
use web_sys::HtmlElement;

/// Apply heading typography. Levels: 1 (largest) through 3.
pub fn heading_text(level: u8) -> impl FnOnce(DomBuilder<HtmlElement>) -> DomBuilder<HtmlElement> {
    move |b| {
        let b = match level {
            1 => b.dwclass!("text-4xl font-bold leading-tight"),
            2 => b.dwclass!("text-2xl font-bold leading-tight"),
            3 => b.dwclass!("text-xl font-semibold leading-snug"),
            _ => b.dwclass!("text-lg font-semibold leading-snug"),
        };
        b.style("color", "var(--ds-color-text)")
    }
}

/// Body text — default paragraph styling.
pub fn body_text() -> impl FnOnce(DomBuilder<HtmlElement>) -> DomBuilder<HtmlElement> {
    |b| {
        b.dwclass!("text-base font-normal leading-normal")
            .style("color", "var(--ds-color-text)")
    }
}

/// Caption text — secondary information, metadata.
pub fn caption_text() -> impl FnOnce(DomBuilder<HtmlElement>) -> DomBuilder<HtmlElement> {
    |b| {
        b.dwclass!("text-sm font-normal leading-normal")
            .style("color", "var(--ds-color-text-muted)")
    }
}

/// Label text — form labels, badges, small UI text.
pub fn label_text() -> impl FnOnce(DomBuilder<HtmlElement>) -> DomBuilder<HtmlElement> {
    |b| {
        b.dwclass!("text-xs font-medium leading-normal")
            .style("color", "var(--ds-color-text-muted)")
    }
}
```

## mixins/surfaces.rs

```rust
use dominator::DomBuilder;
use web_sys::HtmlElement;

/// Standard card surface with themed background and border.
pub fn card_surface() -> impl FnOnce(DomBuilder<HtmlElement>) -> DomBuilder<HtmlElement> {
    |b| {
        b.dwclass!("rounded-lg")
            .style("background", "var(--ds-color-bg-elevated)")
            .style("border", "1px solid var(--ds-color-border-muted)")
            .style("padding", "var(--ds-space-md)")
    }
}

/// Elevated surface with shadow for modals, dropdowns, popovers.
pub fn elevated_surface() -> impl FnOnce(DomBuilder<HtmlElement>) -> DomBuilder<HtmlElement> {
    |b| {
        b.dwclass!("rounded-lg shadow-xl")
            .style("background", "var(--ds-color-bg-elevated)")
            .style("border", "1px solid var(--ds-color-border-muted)")
            .style("padding", "var(--ds-space-lg)")
    }
}
```

## build.rs

```rust
use dominator_css_bindgen::css::generate_rust_bindings_from_file;
use std::path::PathBuf;

fn main() {
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let css_dir = PathBuf::from("resources/css");

    generate_rust_bindings_from_file(
        &css_dir.join("tokens.css"),
        &out_dir.join("tokens.rs"),
    );

    println!("cargo:rerun-if-changed=resources/css/");
}
```

## lib.rs

```rust
#[macro_use]
extern crate dwind_macros;

pub mod theme;
pub mod mixins;

mod tokens_css {
    include!(concat!(env!("OUT_DIR"), "/tokens.rs"));
}

/// Call once at app startup before rendering any components.
pub fn init_design_system() {
    dwind::stylesheet();
    tokens_css::init_styles();
    theme::apply_theme(theme::Theme::Dark);
}
```

## Usage

```rust
use my_design_system::{init_design_system, theme, mixins::typography::*};
use dominator::{html, Dom};

fn app() -> Dom {
    init_design_system();

    html!("div", {
        .style("background", "var(--ds-color-bg)")
        .style("min-height", "100vh")
        .dwclass!("flex flex-col gap-6 p-6")
        .child(html!("h1", {
            .apply(heading_text(1))
            .text("Welcome")
        }))
        .child(html!("p", {
            .apply(body_text())
            .text("This uses your design system tokens.")
        }))
    })
}
```
