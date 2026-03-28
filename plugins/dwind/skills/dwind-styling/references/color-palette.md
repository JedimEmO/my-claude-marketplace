# Dwind Color Palette

All colors available for `bg-{color}-{shade}`, `text-{color}-{shade}`, `border-{color}-{shade}`, `gradient-from-{color}-{shade}`, `gradient-to-{color}-{shade}`.

Opacity modifier: append `/{opacity}` e.g. `bg-blue-500/50` for 50% opacity.

## Colors

### blue
| Shade | Hex |
|-------|-----|
| 50 | Light blue tint |
| 100-400 | Progressive blue |
| 500 | Primary blue |
| 600-900 | Progressive dark blue |
| 950 | Near-black blue |

### green
Standard green scale, 50-950.

### yellow
Standard yellow scale, 50-950.

### orange
Standard orange scale, 50-950.

### red
Standard red scale, 50-950.

### purple
Standard purple scale, 50-950.

### gray
Neutral gray scale, 50-950. Most commonly used for backgrounds and text.

### woodsmoke
Very dark gray with slight warmth. Good for dark mode backgrounds.
- `woodsmoke-950` / `woodsmoke-900` — near-black backgrounds

### bunker
Very dark blue-gray. Deep, rich dark backgrounds.
- `bunker-950` — deepest dark background

### apple
Green-toned color. Good for success states.
- `apple-500`: `#61BD4CFF`
- `apple-700`: `#317621FF`

### candlelight
Yellow/gold-toned. Good for warning states and accents.

### picton-blue
Bright, vivid blue. Good for primary actions and links.

### charm
Pink/rose-toned. Good for accents and highlights.

## Usage Patterns

### Dark mode backgrounds
```rust
.dwclass!("bg-woodsmoke-950")   // Darkest
.dwclass!("bg-bunker-900")      // Very dark
.dwclass!("bg-gray-900")        // Standard dark
.dwclass!("bg-gray-800")        // Elevated surface
```

### Text colors
```rust
.dwclass!("text-white")         // Primary text on dark
.dwclass!("text-gray-400")      // Secondary text on dark
.dwclass!("text-gray-500")      // Muted text on dark
.dwclass!("text-gray-900")      // Primary text on light
```

### Semantic colors
```rust
.dwclass!("bg-apple-500")       // Success
.dwclass!("bg-candlelight-500") // Warning
.dwclass!("bg-red-500")         // Error/danger
.dwclass!("bg-picton-blue-500") // Info/primary
```

### Gradient examples
```rust
.dwclass!("bg-gradient-to-r gradient-from-picton-blue-500 gradient-to-purple-500")
.dwclass!("bg-gradient-to-b gradient-from-gray-900 gradient-to-bunker-950")
```
