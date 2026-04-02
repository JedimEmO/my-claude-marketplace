---
name: dwind-events
description: Use when the user asks about mouse events, keyboard events, click handling, drag interactions, event propagation, stopPropagation, preventDefault, event_with_options, global_event, or handling user input in dominator. Also triggers on "click handler", "mouse event", "keyboard shortcut", "event bubbling", "passive listener", or "pointer events" in a dwind/dominator context.
version: 1.0.0
---

# Dwind Events — Mouse, Keyboard, and Event Handling in Dominator

Handle user interactions in dominator applications. Covers mouse events, keyboard shortcuts, event options, propagation, and common pitfalls.

## Event Registration

### Basic `.event()` — bubble phase, passive

```rust
html!("div", {
    .event(|e: events::Click| {
        web_sys::console::log_1(&"clicked!".into());
    })
})
```

Registers in **bubble phase** with **passive: true** (cannot call `preventDefault`).

### `.event_with_options()` — control phase and preventability

```rust
html!("div", {
    .event_with_options(
        &EventOptions { preventable: true, ..EventOptions::default() },
        |e: events::KeyDown| {
            e.prevent_default(); // only works with preventable: true
        }
    )
})
```

**`EventOptions` fields:**
- `bubbles: true` (default) → bubble phase listener
- `bubbles: false` → **capture phase** listener
- `preventable: true` → non-passive, allows `e.prevent_default()`
- `preventable: false` (default) → passive listener, `preventDefault` will throw a console warning

### `.global_event()` — listen on window

```rust
html!("div", {
    .global_event(|e: events::MouseUp| {
        // Fires even if mouse is released outside this element
    })
})
```

Registers on the **window** object, not the element. Use for:
- Capturing mouseup after a drag started inside the element
- Global keyboard shortcuts
- Detecting clicks outside a popup

## Mouse Events

### Available types

| Event | Type | Notable methods |
|-------|------|-----------------|
| `events::MouseDown` | mousedown | `mouse_x()`, `mouse_y()`, `button()`, `shift_key()`, `ctrl_key()` |
| `events::MouseUp` | mouseup | same |
| `events::MouseMove` | mousemove | `mouse_x()`, `mouse_y()`, `shift_key()`, `ctrl_key()` |
| `events::Click` | click | same |
| `events::Wheel` | wheel | `mouse_x()`, `mouse_y()`, `delta_x()`, `delta_y()`, `delta_z()` |
| `events::ContextMenu` | contextmenu | prevent to disable right-click menu |
| `events::MouseEnter` | mouseenter | does not bubble |
| `events::MouseLeave` | mouseleave | does not bubble |

### Mouse coordinates

`mouse_x()` and `mouse_y()` return **client coordinates** (viewport-relative, integers).

```rust
.event(|e: events::MouseDown| {
    let screen_x = e.mouse_x() as f64;
    let screen_y = e.mouse_y() as f64;
})
```

For world/canvas coordinates, convert using your viewport transform:
```rust
let world_x = (screen_x - pan_x) / zoom;
let world_y = (screen_y - pan_y) / zoom;
```

### Mouse button

`button()` returns `dominator::events::MouseButton`:
```rust
match e.button() {
    events::MouseButton::Left => { /* primary */ }
    events::MouseButton::Middle => { /* pan */ }
    events::MouseButton::Right => { /* context menu */ }
    _ => {}
}
```

### Modifier keys

Available on all mouse events:
```rust
let shift = e.shift_key();  // bool
let ctrl = e.ctrl_key();    // bool — includes meta_key (Cmd on Mac)
```

Note: `ctrl_key()` in dominator already includes `meta_key()` (`Cmd` on Mac). You do NOT need to check both.

## Keyboard Events

### Basic keyboard handler

```rust
html!("div", {
    .attr("tabindex", "0")  // required for div to receive keyboard events
    .style("outline", "none")
    .event_with_options(
        &EventOptions { preventable: true, ..EventOptions::default() },
        |e: events::KeyDown| {
            let key = e.key();      // "a", "Enter", "Escape", "ArrowDown", etc.
            let ctrl = e.ctrl_key();
            let shift = e.shift_key();

            let handled = match key.as_str() {
                "Delete" => { do_delete(); true }
                "z" | "Z" if ctrl && shift => { do_redo(); true }
                "z" | "Z" if ctrl => { do_undo(); true }
                "Escape" => { do_cancel(); true }
                _ => false,
            };

            if handled {
                e.prevent_default();  // prevent browser default (Ctrl+Z = browser undo)
            }
        }
    )
})
```

**Critical:** `event_with_options` with `preventable: true` is required. Without it, the listener is passive and `prevent_default()` throws:
```
Unable to preventDefault inside passive event listener invocation.
```

### `tabindex` requirement

HTML `<div>` elements don't receive keyboard events by default. Add `tabindex="0"` to make them focusable:
```rust
.attr("tabindex", "0")
.style("outline", "none")  // remove focus ring
```

## Drag Interactions

### Pattern: mousedown → global mousemove → global mouseup

```rust
html!("div", {
    .event(|e: events::MouseDown| {
        // Start drag — record initial position
        start_drag(e.mouse_x(), e.mouse_y());
    })
    .global_event(|e: events::MouseMove| {
        // Track drag — fires even outside the element
        if is_dragging() {
            update_drag(e.mouse_x(), e.mouse_y());
        }
    })
    .global_event(|e: events::MouseUp| {
        // End drag — fires even if released outside the element
        if is_dragging() {
            end_drag();
        }
    })
})
```

Use `global_event` for mousemove and mouseup so dragging works when the cursor leaves the element.

### Preventing context menu during right-click drag

```rust
.event_with_options(
    &EventOptions { preventable: true, ..EventOptions::default() },
    |e: events::ContextMenu| {
        e.prevent_default();
    }
)
```

## CRITICAL: stopPropagation Does NOT Work Reliably

**`e.stop_propagation()` on a child element does NOT reliably prevent a parent's `.event()` handler from firing in dominator.**

This was confirmed empirically: a child div calling `e.stop_propagation()` on mousedown did not prevent the parent div's `.event(mousedown)` handler from executing.

### The problem

```rust
// PARENT
html!("div", {
    .event(|e: events::MouseDown| {
        close_popup(); // THIS FIRES even when child calls stopPropagation
    })

    // CHILD
    .child(html!("div", {
        .event(|e: events::MouseDown| {
            e.stop_propagation(); // DOES NOT WORK
            handle_popup_click();
        })
    }))
})
```

### The fix: check event target with `el.closest()`

Instead of relying on propagation, check whether the click target is inside the child element:

```rust
use wasm_bindgen::JsCast;

html!("div", {
    .event(|e: events::MouseDown| {
        // Check if click is inside the popup
        if let Some(target) = e.target() {
            if let Ok(el) = target.dyn_into::<web_sys::Element>() {
                if el.closest("[data-my-popup]").ok().flatten().is_some() {
                    return; // click was inside popup — don't close
                }
            }
        }
        close_popup();
    })

    .child(html!("div", {
        .attr("data-my-popup", "")  // marker attribute for closest() check
        // ... popup content
    }))
})
```

This pattern works for:
- Popup menus that should close on outside click
- Modal dialogs
- Dropdown menus
- Any "click outside to dismiss" interaction

### Why this happens

Dominator uses gloo-events for event registration. The interaction between passive listeners (`preventable: false`, the default) and propagation stopping may differ from standard `addEventListener` behavior. The exact cause is in gloo-events internals and may vary by browser.

**Rule: never rely on `stopPropagation` across dominator elements. Always use target checking.**

## Scroll / Wheel Events

```rust
.event(|e: events::Wheel| {
    let delta = e.delta_y();     // positive = scroll down
    let screen_x = e.mouse_x();  // cursor position during scroll
    let screen_y = e.mouse_y();
    
    // Zoom at cursor position
    let factor = if delta > 0.0 { 1.0 / 1.1 } else { 1.1 };
    zoom_at(screen_x, screen_y, factor);
})
```

`Wheel` extends mouse events — it has `mouse_x()`, `mouse_y()`, `shift_key()`, `ctrl_key()` in addition to `delta_x/y/z()`.

## SVG Events

SVG elements (`svg!()`) receive the same mouse events as HTML elements. But:

### foreignObject event interaction

Events inside `<foreignObject>` (HTML embedded in SVG) may not propagate to SVG parent elements as expected. If you need both SVG-level and HTML-level event handling:

- Use `pointer-events: none` on the foreignObject if it's purely decorative
- Use the `el.closest()` pattern (above) for click-outside detection
- Don't rely on event bubbling across the SVG/HTML boundary

### Hit targets on SVG elements

SVG elements with `fill="none"` don't receive mouse events by default. For invisible hit targets:

```rust
svg!("circle", {
    .attr("r", "15")
    .attr("fill", "transparent")  // transparent, not none — receives events
    .attr("cursor", "pointer")
    .event(|e: events::MouseDown| { ... })
})
```

`fill="transparent"` → receives events. `fill="none"` → does NOT receive events.

## Event Types Reference

All event types are in `dominator::events`:

```rust
use dominator::events;

// Mouse
events::MouseDown, events::MouseUp, events::MouseMove,
events::Click, events::DoubleClick,
events::MouseEnter, events::MouseLeave,
events::ContextMenu, events::Wheel,

// Keyboard
events::KeyDown, events::KeyUp,

// Form
events::Input, events::Change, events::Focus, events::Blur,

// Other
events::Resize, events::Load, events::Error,
```
