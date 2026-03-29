---
name: dwind-reactivity
description: Use when the user asks about state management, signals, Mutable, reactive updates, conditional rendering, signal composition, child_signal, style_signal, broadcast, map_ref, or encounters signal-related compile errors in a dwind/dominator/futures-signals context.
version: 1.0.0
---

# Dwind Reactivity — Signals & State Management

The dwind stack uses `futures-signals` for fine-grained reactivity. State lives in `Mutable<T>` values; the DOM subscribes to changes via signals.

## Mutable<T>

```rust
use futures_signals::signal::Mutable;

let count = Mutable::new(0);
count.set(5);               // Set value
let val = count.get();      // Get current value
let sig = count.signal();   // Get a signal (for primitives implementing Copy)
let sig = count.signal_cloned(); // For non-Copy types (String, Vec, etc.)
let sig = count.signal_ref(|v| v.len()); // Map reference without cloning
```

## Signal Consumption Rule

**A signal can only be consumed once** (`.map()` takes ownership). If you need the same signal in multiple places, use `.broadcast()`:

```rust
let disabled = disabled.broadcast();

// Now call .signal() as many times as needed
.style_signal("opacity", disabled.signal().map(|d| if d { "0.5" } else { "1" }))
.attr_signal("disabled", disabled.signal().map(|d| if d { Some("disabled") } else { None }))
.attr_signal("aria-disabled", disabled.signal().map(|d| if d { Some("true") } else { None }))
```

If you forget `.broadcast()` and use a signal twice, you get a move error.

## DOM Bindings

### text_signal — Reactive text
```rust
.text_signal(count.signal().map(|n| format!("Count: {}", n)))
```

### child_signal — Conditional DOM (returns Option<Dom>)
```rust
.child_signal(is_open.signal().map(|open| {
    if open { Some(html!("div", { .text("Panel content") })) } else { None }
}))
```

### style_signal — Reactive inline styles
```rust
.style_signal("opacity", is_visible.signal().map(|v| if v { "1" } else { "0" }))
```

### attr_signal — Reactive attributes (returns Option<&str>)
```rust
.attr_signal("disabled", disabled.signal().map(|d| if d { Some("disabled") } else { None }))
```

### visible_signal — Show/hide via CSS display
```rust
.visible_signal(is_visible.signal())
```

### dwclass_signal! — Reactive utility classes
```rust
.dwclass_signal!("bg-blue-500", is_active.signal())
```

## Combining Signals with map_ref!

When a value depends on multiple signals:

```rust
use futures_signals::map_ref;

.style_signal("box-shadow", {
    map_ref! {
        let valid = is_valid.signal(),
        let focused = is_focused.signal() => {
            if !*valid { "var(--shadow-error)" }
            else if *focused { "var(--shadow-focus)" }
            else { "var(--shadow)" }
        }
    }
})
```

## SignalExt Combinators

```rust
use futures_signals::signal::SignalExt;

signal.map(|v| v + 1)           // Transform
not(bool_signal)                 // Negate
and(sig_a, sig_b)                // Logical AND
or(sig_a, sig_b)                 // Logical OR
signal.for_each(|v| async { })  // Side effect
signal.boxed_local()             // Type-erase for trait objects
```

## Reactive Lists

```rust
use futures_signals::signal_vec::{MutableVec, SignalVecExt};

let items = MutableVec::new();
items.lock_mut().push_cloned("new item".to_string());

html!("ul", {
    .children_signal_vec(items.signal_vec_cloned().map(|item| {
        html!("li", { .text(&item) })
    }))
})
```

## Programmatic Responsive Behavior

```rust
use dwind::prelude::media_queries::{breakpoint_active_signal, Breakpoint};

let is_desktop = breakpoint_active_signal(Breakpoint::Medium);

html!("div", {
    .child_signal(is_desktop.map(|desktop| {
        if desktop { Some(desktop_nav()) } else { Some(mobile_nav()) }
    }))
})
```

## Decision Tree: Which Reactive Binding?

- **Structural changes** (add/remove DOM nodes): `child_signal` / `children_signal_vec`
- **Visual changes** (colors, opacity, size): `dwclass_signal!` / `style_signal`
- **Simple show/hide**: `visible_signal` (keeps DOM alive, toggles `display`)
- **Text updates**: `text_signal`
- **Attribute changes**: `attr_signal`

## Critical Gotchas

### style_signal must NEVER return empty string

Dominator panics in debug builds on empty style values:

```rust
// BAD — panics when size isn't Small
.style_signal("border-radius", size.signal().map(|s| match s {
    Size::Small => "4px",
    _ => "",  // PANIC!
}))

// GOOD — every branch returns a valid CSS value
.style_signal("border-radius", size.signal().map(|s| match s {
    Size::Small => "4px",
    Size::Medium => "8px",
    Size::Large => "12px",
}))
```

### Vendor-prefixed CSS needs array syntax

```rust
// BAD — panics if browser doesn't support prefix
.style("-webkit-backdrop-filter", "blur(8px)")

// GOOD — tries each name, succeeds if any works
.style(["backdrop-filter", "-webkit-backdrop-filter"], "blur(8px)")
```

### CSS visibility vs conditional DOM destruction

Prefer CSS visibility over `child_signal` when content has its own signals:

```rust
// PROBLEMATIC — content signal consumed on creation, destroyed on close, can't recreate
.child_signal(open.signal().map(move |is_open| {
    if is_open { Some(panel_with_content_signal) } else { None }
}))

// BETTER — panel always in DOM, CSS controls visibility
.child(html!("div", {
    .style_signal("opacity", open.signal().map(|o| if o { "1" } else { "0" }))
    .style_signal("pointer-events", open.signal().map(|o| if o { "auto" } else { "none" }))
    .child_signal(content)  // consumed once, lives forever
}))
```

### Never use `return` inside map_ref!

The macro expansion makes `return` exit the wrong scope:

```rust
// BAD — type mismatch with Poll
map_ref! { let a = sig => { if *a { return "yes"; } "no" } }

// GOOD — use if/else expression
map_ref! { let a = sig => { if *a { "yes" } else { "no" } } }
```

### Box<dyn Fn()> is not Clone — use Rc

```rust
let on_close = std::rc::Rc::new(on_close);
.event({ let on_close = on_close.clone(); move |_: events::Click| { (on_close)(); } })
.global_event({ let on_close = on_close.clone(); move |e: events::KeyDown| {
    if e.key() == "Escape" { (on_close)(); }
}})
```

### <label> doesn't have :disabled

Use explicit style signals for disabled state on `<label>` elements:

```rust
.style_signal("opacity", disabled.signal().map(|d| if d { "0.5" } else { "1" }))
.style_signal("pointer-events", disabled.signal().map(|d| if d { "none" } else { "auto" }))
```
