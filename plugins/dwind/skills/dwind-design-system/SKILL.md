---
name: dwind-design-system
description: Use when the user asks about design tokens, design system architecture, spacing scales, type scales, color systems, semantic tokens, component spacing conventions, vertical rhythm, dark/light theme token mapping, accessibility contrast ratios, or organizing a design system crate in a dwind/dominator context. Also triggers when the user mentions token hierarchy, baseline grid, or design system structure.
version: 1.0.0
---

# Dwind Design System — Tokens, Scales & Conventions

A design system is a set of deliberate decisions about tokens, scales, and conventions that create visual consistency across an application. This skill covers the *architecture* of a design system built on dwind — what to define, how to layer it, and what constraints to enforce.

For utility class mechanics and the `dwclass!` macro, see **dwind-styling**. For component patterns and the `#[component]` macro, see **dwind-component**. For project scaffolding and build pipeline, see **dwind-project-setup**.

## Design Token Architecture

Structure tokens in three layers. Each layer references the one below it, creating a hierarchy that makes theme switching trivial and naming intentional.

**Layer 1 — Primitive tokens**: raw values. These map directly to dwind's built-in palette and spacing scale. Name them after what they *are*.

```css
/* tokens.css — primitives */
.color-blue-500 { --ds-color-blue-500: #3b82f6; }
.color-gray-900 { --ds-color-gray-900: #111827; }
.space-2 { --ds-space-2: 8px; }
.space-4 { --ds-space-4: 16px; }
.text-base { --ds-text-base: 16px; }
```

**Layer 2 — Semantic tokens**: purpose-based aliases. Name them after what they *do*. These are what theme switching swaps.

```css
/* tokens.css — semantic (dark mode default) */
:root {
  --ds-color-bg: var(--ds-color-gray-900);
  --ds-color-bg-elevated: var(--ds-color-gray-800);
  --ds-color-text: var(--ds-color-gray-50);
  --ds-color-primary: var(--ds-color-blue-500);
  --ds-space-content-gap: var(--ds-space-4);
}
```

```css
/* Light mode overrides — only remap semantic tokens */
.light {
  --ds-color-bg: var(--ds-color-gray-50);
  --ds-color-bg-elevated: var(--ds-color-white);
  --ds-color-text: var(--ds-color-gray-900);
}
```

**Layer 3 — Component tokens** (optional): scoped overrides for a specific component. Use only when a component has internal values that differ from the semantic defaults.

```css
.card-tokens {
  --card-padding: var(--ds-space-content-gap);
  --card-radius: 12px;
}
```

Theme switching works by redefining semantic tokens — primitives and component tokens stay untouched. See **dwind-styling** for the `apply_theme_to_root()` function that applies CSS variables at runtime.

## Spacing System

Dwind's spacing scale uses a 4px base unit: `gap-1` = 4px, `gap-2` = 8px, `gap-4` = 16px. The full scale goes from 0 to 96 (384px). That's too many choices — constrain it.

Define a semantic spacing scale that picks 6-7 values from dwind's range:

| Token | Value | Dwind class | Use for |
|-------|-------|-------------|---------|
| `--ds-space-xs` | 4px | `gap-1`, `p-1` | Icon-to-label gaps, tight inline spacing |
| `--ds-space-sm` | 8px | `gap-2`, `p-2` | Related element spacing, compact padding |
| `--ds-space-md` | 16px | `gap-4`, `p-4` | Default content gaps, card padding |
| `--ds-space-lg` | 24px | `gap-6`, `p-6` | Section separation, generous padding |
| `--ds-space-xl` | 32px | `gap-8`, `p-8` | Major section breaks |
| `--ds-space-2xl` | 48px | `gap-12`, `p-12` | Page-level margins, hero spacing |

**When to use `dwclass!` vs CSS vars**: use `dwclass!("gap-4 p-4")` for layout — it's shorter and compiles to zero-cost CSS. Use `.style("padding", "var(--ds-space-md)")` when the value needs to be themeable or referenced by component tokens.

**Vertical rhythm**: pick a baseline line-height (24px / `leading-6` is a good default). Ensure vertical margins and paddings are multiples of this baseline. This creates a predictable visual rhythm — elements align to an invisible grid.

## Color System

Build semantic colors on top of dwind's palette. Define these categories:

**Backgrounds**: `--ds-color-bg`, `--ds-color-bg-elevated`, `--ds-color-bg-muted`, `--ds-color-bg-inverted`

**Text**: `--ds-color-text`, `--ds-color-text-muted`, `--ds-color-text-inverted`

**Interactive**: `--ds-color-primary`, `--ds-color-primary-hover`, `--ds-color-secondary`

**Borders**: `--ds-color-border`, `--ds-color-border-muted`

**Status**: `--ds-color-success`, `--ds-color-warning`, `--ds-color-error`, `--ds-color-info`

### Dark/Light Mapping

| Semantic token | Dark (default) | Light |
|----------------|---------------|-------|
| `--ds-color-bg` | `gray-900` | `gray-50` |
| `--ds-color-bg-elevated` | `gray-800` | `white` |
| `--ds-color-text` | `gray-50` | `gray-900` |
| `--ds-color-text-muted` | `gray-400` | `gray-500` |
| `--ds-color-primary` | `blue-500` | `blue-600` |
| `--ds-color-border` | `gray-700` | `gray-200` |
| `--ds-color-error` | `red-400` | `red-600` |

Note how some semantic tokens map to different shades in each theme — `blue-500` has sufficient contrast on dark backgrounds, but `blue-600` is needed on light backgrounds. Always verify contrast.

### Accessibility Contrast Requirements

- **Normal text** (text-sm through text-base): 4.5:1 contrast ratio against background (WCAG AA)
- **Large text** (text-xl and above, or bold text-lg+): 3:1 contrast ratio
- **UI components** (borders, icons, focus indicators): 3:1 contrast ratio

Test with browser dev tools — inspect an element, check the contrast ratio in the color picker. Design the tokens to pass from the start rather than fixing failures later.

## Typography System

Map dwind's type scale to semantic roles instead of using raw sizes everywhere:

| Role | Dwind class | Weight | Line height | Use for |
|------|-------------|--------|-------------|---------|
| Heading 1 | `text-4xl` | `font-bold` | `leading-tight` | Page titles |
| Heading 2 | `text-2xl` | `font-bold` | `leading-tight` | Section headers |
| Heading 3 | `text-xl` | `font-semibold` | `leading-snug` | Subsection headers |
| Body | `text-base` | `font-normal` | `leading-normal` | Paragraph text |
| Caption | `text-sm` | `font-normal` | `leading-normal` | Secondary info |
| Label | `text-xs` | `font-medium` | `leading-normal` | Form labels, badges |

Encode these as mixin functions so every heading looks the same:

```rust
pub fn heading_text(level: u8) -> impl FnOnce(DomBuilder<HtmlElement>) -> DomBuilder<HtmlElement> {
    move |b| match level {
        1 => b.dwclass!("text-4xl font-bold leading-tight"),
        2 => b.dwclass!("text-2xl font-bold leading-tight"),
        3 => b.dwclass!("text-xl font-semibold leading-snug"),
        _ => b.dwclass!("text-lg font-semibold leading-snug"),
    }
    .style("color", "var(--ds-color-text)")
}
```

**Responsive headings**: scale up on larger screens. `dwclass!("text-2xl @md:text-4xl")` makes a heading that's `text-2xl` on mobile and `text-4xl` on desktop.

## Component Spacing Conventions

**Rule: components never set their own outer margins.** The parent controls spacing.

Why: a card that sets `m-b-4` breaks when placed in a flex container with `gap-6`. Margins create coupling between a component and its context.

**Correct pattern**: parent uses `gap-{n}` or `space-y-{n}`, children have zero margin:

```rust
// Page layout — parent controls all spacing
html!("div", {
    .dwclass!("flex flex-col gap-6 p-6")
    .child(header_component())     // no outer margin
    .child(content_card())         // no outer margin
    .child(footer_component())     // no outer margin
})
```

**Internal padding is fine**: a card can set `p-4` because that's within its own boundary.

**Escape hatch**: the `apply` extension point on `#[component]` structs lets consumers add context-specific styling when needed:

```rust
my_card!({
    .title("Settings")
    .apply(|b| b.dwclass!("m-t-2"))  // consumer adds margin for this specific context
})
```

Use this sparingly. If you find yourself adding margins via `apply` everywhere, the parent layout isn't doing its job.

## Design System Crate Structure

Expand on the brief sketch in **dwind-project-setup** with a full layout:

```
crates/my-design-system/
├── Cargo.toml
├── build.rs                       # CSS codegen (tokens.css → tokens.rs)
├── resources/css/
│   └── tokens.css                 # All design tokens (primitive + semantic)
└── src/
    ├── lib.rs                     # Stylesheet init, re-exports
    ├── tokens_css.rs              # Generated — include! from OUT_DIR
    ├── theme/
    │   ├── mod.rs                 # Theme enum (Dark, Light), apply_theme()
    │   └── palettes.rs            # Palette definitions for each theme
    ├── mixins/
    │   ├── mod.rs
    │   ├── typography.rs          # heading_text(), body_text(), caption_text()
    │   └── surfaces.rs            # card_surface(), elevated_surface()
    └── components/
        └── mod.rs                 # Design system components
```

The `lib.rs` calls `dwind::stylesheet()` and injects the design system's generated token stylesheet. Application crates depend on the design system crate — not on dwind directly — to enforce that all styling goes through the token layer.

```rust
// lib.rs
#[macro_use]
extern crate dwind_macros;

pub mod theme;
pub mod mixins;

mod tokens_css {
    include!(concat!(env!("OUT_DIR"), "/tokens.rs"));
}

pub fn init_design_system() {
    dwind::stylesheet();
    tokens_css::init_styles();
}
```

See `references/design-system-template.md` for a complete, copy-pasteable starter.

## Quick Decision Checklist

When starting a design system, decide these up front:

1. **Baseline unit**: 4px (matches dwind's scale)
2. **Spacing scale**: curate 6-7 named sizes (xs through 2xl) from dwind's range
3. **Primary palette**: pick primary, secondary, and accent colors from dwind's palette
4. **Semantic colors**: define bg, text, primary, border, and status tokens for both themes
5. **Contrast**: verify 4.5:1 for body text, 3:1 for large text and UI elements
6. **Type scale**: assign heading/body/caption/label roles from dwind's text-xs through text-9xl
7. **Component spacing**: margin-free components, parent-controlled via gap/space
8. **Crate boundary**: single design-system crate re-exporting tokens, theme, mixins, and components
