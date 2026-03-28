---
name: dwind-styling
description: This skill should be used when the user asks about styling, CSS classes, colors, spacing, layout, responsive design, hover/focus states, animations, visual appearance, or theming in a dwind/dominator context. Also triggers when the user mentions dwclass, utility classes, or breakpoints.
tools: Read, Glob, Grep, Edit, Write, Bash
---

# Dwind Styling — Utility Classes & Visual Design

Dwind provides Tailwind-like utility classes that compile into the WASM binary at build time. All styling uses procedural macros — no runtime CSS parsing.

## dwclass! — Basic Usage

Apply utility classes to dominator elements:

```rust
html!("div", {
    .dwclass!("flex gap-4 p-4")           // Multiple classes in one call
    .dwclass!("bg-gray-900 text-white")   // Chain multiple calls
})
```

**Important**: `dwclass!` only accepts string literals. No variables or dynamic strings.

### Two-parameter form (inside closures)

```rust
html!("div", {
    .apply(|b| {
        match variant {
            0 => dwclass!(b, "bg-red-500"),
            1 => dwclass!(b, "bg-blue-500"),
            _ => dwclass!(b, "bg-gray-500"),
        }
    })
})
```

## dwclass_signal! — Reactive Styling

Toggle classes based on signals:

```rust
let is_active = Mutable::new(false);
html!("div", {
    .dwclass!("p-4 rounded")
    .dwclass_signal!("bg-blue-500", is_active.signal())      // Applied when true
    .dwclass_signal!("opacity-50", not(is_active.signal()))   // Applied when false
})
```

## dwgenerate! — Custom Reusable Classes

Pre-declare reusable class combinations:

```rust
dwgenerate!("btn-primary", "hover:bg-blue-600 active:scale-95");
html!("button", {
    .dwclass!("btn-primary px-4 py-2 bg-blue-500")
})
```

Arbitrary values:

```rust
.dwclass!("padding-[20px]")     // Custom spacing
.dwclass!("bg-[#ff5500]")       // Custom color
```

## Responsive Breakpoints

Mobile-first. Prefix classes with `@breakpoint:`:

| Prefix | Width | Description |
|--------|-------|-------------|
| `@xs:` | < 640px | Default (no prefix needed) |
| `@sm:` | >= 640px | Small screens |
| `@md:` | >= 1280px | Medium screens |
| `@lg:` | >= 1920px | Large screens |
| `@xl:` | >= 2560px | Extra large |
| `@<sm:` | < 640px | Less than small |

```rust
.dwclass!("flex-col @sm:flex-row")             // Column mobile, row desktop
.dwclass!("gap-2 @md:gap-4 @lg:gap-8")         // Increasing gap
.dwclass!("@<sm:hidden @sm:block")             // Hidden on mobile
```

Custom media queries:

```rust
.dwclass!("@((max-width: 700px)):bg-red-500")
```

## Pseudo-Classes

```rust
.dwclass!("hover:bg-blue-600")
.dwclass!("focus:ring-2 focus:ring-blue-400")
.dwclass!("active:scale-95")
.dwclass!("disabled:opacity-50 disabled:cursor-not-allowed")
.dwclass!("nth-child(2):bg-gray-800")
.dwclass!("nth-child(odd):bg-gray-900")
.dwclass!("is(.selected):font-bold")
```

## Variant Selectors (Child Styling)

Apply styles to child elements with `[selector]:class`:

```rust
.dwclass!("[& > *]:p-2")                       // All direct children
.dwclass!("[> span]:text-blue-500")            // Direct span children
.dwclass!("[& > *]:nth-child(2):bg-red-500")   // Second direct child
.dwclass!("[& *]:w-full")                       // All descendants
.dwclass!("[& > button]:hover:bg-blue-600")    // Direct buttons on hover
```

## Color Opacity

```rust
.dwclass!("bg-blue-500/50")     // 50% opacity background
.dwclass!("text-white/75")      // 75% opacity text
```

## Common Patterns

### Centered Container
```rust
.dwclass!("flex justify-center align-items-center h-full")
```

### Card Layout
```rust
.dwclass!("p-4 bg-gray-900 rounded-lg shadow-lg border border-gray-800")
```

### Responsive Grid
```rust
.dwclass!("grid grid-cols-1 @sm:grid-cols-2 @md:grid-cols-3 gap-4")
```

### Button with States
```rust
.dwclass!("px-4 py-2 bg-blue-500 rounded")
.dwclass!("hover:bg-blue-600 active:scale-95")
.dwclass!("disabled:opacity-50 disabled:cursor-not-allowed")
```

### Theme-Aware (Light/Dark)
```rust
// Parent element has "light" class for light mode
.dwclass!("bg-gray-900 is(.light):bg-gray-100")
.dwclass!("text-white is(.light):text-gray-900")
```

## Glass Visual Depth Tips

Flat semi-transparent backgrounds look like colored rectangles. Add depth:

1. **Bevel highlight**: `inset 0 0.5px 0 0 rgba(255,255,255,0.1)` box-shadow simulates light catching the top edge
2. **Light gradient overlay**: `linear-gradient(to bottom, rgba(255,255,255,0.06), transparent)` on top of background
3. **No hard borders**: Use box-shadow rings (`0 0 0 2px var(--accent-muted)`) instead of `border-color` — shadows are anti-aliased

```rust
// Bevel + shadow combo
.style("box-shadow", "inset 0 0.5px 0 0 rgba(255,255,255,0.1), 0 4px 16px rgba(0,0,0,0.15)")

// Light gradient on elevated surfaces
.style("background", "\
    linear-gradient(to bottom, rgba(255,255,255,0.06), transparent 50%), \
    var(--my-bg-elevated)")
```

## Color Palette

Named colors with shades 50–950: `blue`, `green`, `yellow`, `orange`, `red`, `purple`, `gray`, `woodsmoke`, `bunker`, `apple`, `candlelight`, `picton-blue`

Usage: `bg-{color}-{shade}`, `text-{color}-{shade}`, `border-{color}-{shade}`

Read `references/color-palette.md` for all color values and `references/utility-classes.md` for the complete class reference.

## Gradients

```rust
.dwclass!("bg-gradient-to-r gradient-from-blue-500 gradient-to-purple-500")
.dwclass!("linear-gradient-135 gradient-from-gray-900 gradient-to-gray-800")
```

Directions: `bg-gradient-to-{t|tr|r|br|b|bl|l|tl}`, angles: `linear-gradient-{0|45|90|135|180}`
