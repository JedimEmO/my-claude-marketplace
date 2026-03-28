# Dwind Component Catalog

Complete reference for all available components in dwui.

## DWUI Components

Source: `/home/mmy/repos/oss/dominator-css-bindgen/crates/dwui/src/components/`

### Button (`button!`)

| Prop | Type | Signal | Default |
|------|------|--------|---------|
| `content` | `Option<Dom>` | Yes | `None` |
| `on_click` | `dyn Fn(events::Click)` | No | no-op |
| `disabled` | `bool` | Yes | `false` |
| `button_type` | `ButtonType` | Yes | `ButtonType::Flat` |

Variants: `ButtonType::Flat`, `ButtonType::Border`

```rust
button!({
    .content(text("Click me"))
    .on_click(|_| { /* handle */ })
    .disabled_signal(is_disabled.signal())
})
```

### Modal (`modal!`)

| Prop | Type | Signal | Default |
|------|------|--------|---------|
| `content` | `Option<Dom>` | Yes | `None` |
| `open` | `bool` | Yes | `false` |
| `on_close` | `dyn Fn()` | No | no-op |
| `size` | `ModalSize` | Yes | `ModalSize::Medium` |
| `close_on_backdrop_click` | `bool` | Yes | `true` |

Variants: `ModalSize::Small`, `Medium`, `Large`, `Full`

```rust
modal!({
    .content(text("Modal body"))
    .open_signal(is_open.signal())
    .on_close(|| { is_open.set(false); })
})
```

### TextInput (`text_input!`)

| Prop | Type | Signal | Default |
|------|------|--------|---------|
| `value` | `dyn InputValueWrapper` | No | `Mutable::new("")` |
| `is_valid` | `ValidationResult` | Yes | `Valid` |
| `label` | `String` | Yes | `""` |
| `on_submit` | `dyn FnMut()` | No | no-op |
| `input_type` | `TextInputType` | Yes | `Text` |
| `claim_focus` | `bool` | No | `false` |

```rust
text_input!({
    .value(Box::new(text_mutable.clone()))
    .label_signal(always("Username"))
    .input_type(TextInputType::Text)
})
```

### Select (`select!`)

| Prop | Type | Signal | Default |
|------|------|--------|---------|
| `value` | `dyn InputValueWrapper` | No | `Mutable::new("")` |
| `options` | `Vec<(String, String)>` | SignalVec | `[]` |
| `label` | `String` | Yes | `""` |
| `is_valid` | `ValidationResult` | Yes | `Valid` |

```rust
select!({
    .value(Box::new(selected.clone()))
    .options_signal_vec(options_signal)
    .label_signal(always("Choose"))
})
```

### Slider (`slider!`)

| Prop | Type | Signal | Default |
|------|------|--------|---------|
| `value` | `dyn InputValueWrapper` | No | `Mutable::new("")` |
| `min` | `f32` | Yes | `0.0` |
| `max` | `f32` | Yes | `100.0` |
| `step` | `f32` | Yes | `1.0` |
| `label` | `String` | Yes | `""` |

```rust
slider!({
    .value(Box::new(val.clone()))
    .min_signal(always(0.0))
    .max_signal(always(100.0))
})
```

### Card (`card!`)

| Prop | Type | Signal | Default |
|------|------|--------|---------|
| `content` | `Dom` | Yes | required |
| `scheme` | `ColorScheme` | Yes | `ColorScheme::Void` |

Variants: `ColorScheme::Primary`, `Secondary`, `Void`

```rust
card!({
    .content_signal(always(text("Card body")))
    .scheme_signal(always(ColorScheme::Primary))
})
```

### Heading (`heading!`)

| Prop | Type | Signal | Default |
|------|------|--------|---------|
| `content` | `Dom` | Yes | required |
| `text_size` | `TextSize` | Yes | `TextSize::ExtraLarge` |

```rust
heading!({
    .content_signal(always(text("Title")))
    .text_size_signal(always(TextSize::Large))
})
```

### List (`pretty_list!`)

| Prop | Type | Signal | Default |
|------|------|--------|---------|
| `items` | `Vec<Dom>` | SignalVec | `[]` |
| `selected_index` | `Option<usize>` | Yes | `None` |
| `item_click_handler` | `dyn Fn(usize)` | No | no-op |

```rust
pretty_list!({
    .items_signal_vec(items_signal)
    .selected_index_signal(selected.signal())
    .item_click_handler(|index| { /* handle */ })
})
```

---

## Common Patterns

### Signal Props
All `#[signal]` props accept both static and reactive values:
- Static: `.prop(value)`
- Reactive: `.prop_signal(signal)`

### Validation
```rust
pub enum ValidationResult {
    Valid,
    Invalid { message: String },
}
```

### Custom Styling via apply
```rust
button!({
    .content(text("Custom"))
    .apply(|b| b.dwclass!("border border-blue-500"))
})
```
