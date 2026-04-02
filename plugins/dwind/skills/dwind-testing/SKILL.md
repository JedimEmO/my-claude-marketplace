---
name: dwind-testing
description: Use when the user asks about testing dwind/dominator WASM components, writing wasm-bindgen-test tests, DOM isolation between tests, testing reactive signals, or debugging rendering issues in headless browsers. Also triggers on "test my component", "wasm test", "DOM test", "browser test", or "test isolation".
version: 1.0.0
---

# Dwind Testing — wasm-bindgen-test Patterns

Write browser-based tests for dwind/dominator components using `wasm-bindgen-test`.

## Setup

### Cargo.toml

```toml
[dev-dependencies]
wasm-bindgen-test = "0.3"
js-sys = "0.3"
wasm-bindgen-futures = "0.4"

[dependencies]
# Ensure web-sys has enough features for test queries
web-sys = { version = "0.3", features = [
    "Document", "Element", "HtmlElement", "NodeList",
    "DomRect", "Window", "console",
] }
```

### Running Tests

```bash
# Firefox (recommended — more stable in headless)
wasm-pack test --headless --firefox crates/my-crate

# Chrome
wasm-pack test --headless --chrome crates/my-crate

# With output (see console.log and panic messages)
wasm-pack test --headless --firefox crates/my-crate -- --nocapture

# Single test
wasm-pack test --headless --firefox crates/my-crate -- --nocapture test_my_thing
```

## Critical: DOM Isolation Between Tests

**All wasm-bindgen-test tests share the same `document.body`.** DOM elements from one test persist into the next unless explicitly removed. This causes:
- Element count assertions failing (accumulating elements)
- `querySelector` finding elements from previous tests
- Signal subscriptions from old tests interfering with new ones

### The TestContainer Pattern

Every test that renders DOM must use an isolated container that cleans up on drop:

```rust
use wasm_bindgen::JsCast;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

/// Isolated test container. Removed from DOM on drop.
struct TestContainer {
    element: web_sys::Element,
}

impl TestContainer {
    fn new() -> Self {
        let doc = web_sys::window().unwrap().document().unwrap();
        let el = doc.create_element("div").unwrap();
        // Give it a real size so layout works correctly
        el.set_attribute("style",
            "position:absolute;left:0;top:0;width:800px;height:600px"
        ).unwrap();
        doc.body().unwrap().append_child(&el).unwrap();
        Self { element: el }
    }

    fn dom_element(&self) -> web_sys::HtmlElement {
        self.element.clone().dyn_into().unwrap()
    }

    /// Query within this container only — never polluted by other tests.
    fn query_all(&self, selector: &str) -> web_sys::NodeList {
        self.element.query_selector_all(selector).unwrap()
    }

    fn query(&self, selector: &str) -> Option<web_sys::Element> {
        self.element.query_selector(selector).unwrap()
    }
}

impl Drop for TestContainer {
    fn drop(&mut self) {
        self.element.remove();
    }
}
```

### Usage

```rust
#[wasm_bindgen_test]
async fn test_my_component() {
    let tc = TestContainer::new();

    // Render INTO the container, not into body
    dominator::append_dom(&tc.dom_element(), my_component());

    wait_frame().await;

    // Query scoped to this test's container only
    let buttons = tc.query_all("button");
    assert_eq!(buttons.length(), 1);
}
```

**Rules:**
- Always capture the container: `let _tc = ...` (underscore prefix keeps it alive without using it)
- If a test queries the DOM, use `tc.query()` / `tc.query_all()`, NOT `document.query_selector()`
- If a test only checks signal/state values (no DOM queries), still capture `_tc` so the DOM is cleaned up

## Waiting for Rendering

Dominator batches DOM updates asynchronously. After changing a `Mutable` or appending DOM, you must wait before reading the result.

### wait_frame helper

```rust
async fn wait_frame() {
    let promise = js_sys::Promise::new(&mut |resolve, _| {
        web_sys::window().unwrap()
            .request_animation_frame(&resolve).unwrap();
    });
    wasm_bindgen_futures::JsFuture::from(promise).await.unwrap();
}

async fn wait_frames(n: usize) {
    for _ in 0..n {
        wait_frame().await;
    }
}
```

### When to wait

| Scenario | Frames to wait |
|----------|---------------|
| After `dominator::append_dom()` | 1 |
| After changing a `Mutable` that drives `style_signal` / `text_signal` | 1 |
| After `children_signal_vec` adds/removes elements | 1–2 |
| After `requestAnimationFrame` callback (e.g., DOM measurement) | 2–3 |
| After `full_sync` that rebuilds entire DOM tree | 3 |

## Testing Reactive Signals

### Test that a Mutable change propagates to DOM

```rust
#[wasm_bindgen_test]
async fn test_reactive_text() {
    let tc = TestContainer::new();
    let label = Mutable::new("Hello".to_string());

    dominator::append_dom(&tc.dom_element(), html!("span", {
        .attr("data-testid", "label")
        .text_signal(label.signal_cloned())
    }));
    wait_frame().await;

    let el = tc.query("[data-testid=label]").unwrap();
    assert_eq!(el.text_content().unwrap(), "Hello");

    label.set("World".to_string());
    wait_frame().await;

    assert_eq!(el.text_content().unwrap(), "World");
}
```

### Test that MutableVec drives children_signal_vec

```rust
#[wasm_bindgen_test]
async fn test_reactive_list() {
    let tc = TestContainer::new();
    let items = MutableVec::new();

    dominator::append_dom(&tc.dom_element(), html!("ul", {
        .children_signal_vec(items.signal_vec_cloned().map(|item: String| {
            html!("li", { .text(&item) })
        }))
    }));
    wait_frame().await;

    assert_eq!(tc.query_all("li").length(), 0);

    items.lock_mut().push_cloned("First".to_string());
    wait_frames(2).await;

    assert_eq!(tc.query_all("li").length(), 1);
}
```

## Testing DOM Measurements

When testing code that reads `getBoundingClientRect()`, the container must have real dimensions. The `TestContainer` sets `width:800px;height:600px` for this reason.

**Gotcha:** Headless browsers may report `(0, 0)` for elements that aren't visible. Ensure:
- The container has explicit dimensions
- Elements use `position: absolute` with explicit `left`/`top` for predictable layout
- Don't rely on CSS Flexbox/Grid sizing in tests — use explicit pixel values

### Converting screen ↔ world coordinates in tests

If your component uses a pan/zoom transform container:

```rust
// Read an element's world position from its screen position
let el = tc.query("[data-my-element]").unwrap();
let rect = el.get_bounding_client_rect();
let center_x = rect.left() + rect.width() / 2.0;
let center_y = rect.top() + rect.height() / 2.0;

// Find the transform container
let vp = tc.query("[data-viewport-inner]").unwrap();
let vp_rect = vp.get_bounding_client_rect();
let zoom = my_zoom_signal.get();

let world_x = (center_x - vp_rect.left()) / zoom;
let world_y = (center_y - vp_rect.top()) / zoom;
```

## Testing User Interactions

For components that handle mouse/keyboard events, test at the signal/state level rather than simulating DOM events. DOM event simulation in wasm-bindgen-test is unreliable.

```rust
// GOOD — test the handler directly
gs.handle_input(InputEvent::MouseDown {
    screen: Vec2::new(100.0, 100.0),
    world: Vec2::new(100.0, 100.0),
    button: MouseButton::Left,
    modifiers: Modifiers::default(),
});
assert!(matches!(gs.state(), SomeState::Dragging { .. }));

// BAD — dispatching synthetic DOM events is fragile
let event = web_sys::MouseEvent::new("mousedown").unwrap();
element.dispatch_event(&event).unwrap(); // unreliable
```

## Common Pitfalls

### 1. Forgotten container capture

```rust
// BAD — container dropped immediately, DOM removed before assertions
async fn test_bad() {
    render_into_container();  // container dropped here
    wait_frame().await;
    // DOM is already gone!
}

// GOOD — container kept alive
async fn test_good() {
    let _tc = render_into_container();
    wait_frame().await;
    // DOM still exists
}
```

### 2. Querying global document instead of container

```rust
// BAD — finds elements from ALL tests
let els = document.query_selector_all("button").unwrap();

// GOOD — scoped to this test
let els = tc.query_all("button");
```

### 3. Not waiting enough frames after complex operations

```rust
// BAD — MutableVec change + DOM measurement in same frame
items.lock_mut().push_cloned(value);
let count = tc.query_all("li").length(); // still 0!

// GOOD — wait for dominator to flush
items.lock_mut().push_cloned(value);
wait_frames(2).await;
let count = tc.query_all("li").length(); // correct
```

### 4. Asserting exact element counts across shared DOM

```rust
// BAD — fragile if test order changes
assert_eq!(tc.query_all("[data-node]").length(), 2);

// BETTER — use >= for existence checks, == only within isolated container
assert!(tc.query_all("[data-node]").length() >= 2);
// Or with TestContainer: exact counts are safe since container is isolated
assert_eq!(tc.query_all("[data-node]").length(), 2); // ✓ safe with TestContainer
```
