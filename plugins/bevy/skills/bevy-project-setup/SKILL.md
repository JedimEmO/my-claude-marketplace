---
name: bevy-project-setup
description: Use when the user asks to create a new Bevy project, scaffold a game, configure Cargo.toml for Bevy, set up fast compile times, configure dynamic linking, add Bevy feature flags, or asks about Bevy project structure and build optimization.
tools: Read, Glob, Grep, Edit, Write, Bash
---

# Bevy Project Setup — Scaffolding & Build Optimization

> For general Rust workspace conventions, crate boundaries, and Cargo.toml management, see the **rust-project-setup** skill first. This skill covers Bevy-specific additions on top of that foundation.

## Project Structure

A standard Bevy game project follows this layout:

```
my_game/
├── Cargo.toml
├── .cargo/
│   └── config.toml          # Linker & fast-compile settings
├── src/
│   ├── main.rs              # App entry point
│   ├── plugins/
│   │   ├── mod.rs
│   │   ├── camera.rs        # Camera plugin
│   │   ├── player.rs        # Player plugin
│   │   └── ui.rs            # UI plugin
│   ├── components/
│   │   └── mod.rs           # Shared components
│   ├── resources/
│   │   └── mod.rs           # Shared resources
│   └── systems/
│       └── mod.rs           # Standalone systems
├── assets/
│   ├── textures/
│   ├── models/
│   ├── audio/
│   └── fonts/
└── README.md
```

Each gameplay domain gets its own plugin module in `src/plugins/`. Components and resources that are shared across plugins live in their own top-level modules. Keep plugins focused: one responsibility per plugin.

## Cargo.toml

Use a workspace-based setup. The game crate depends on Bevy with explicit feature selection:

```toml
[workspace]
resolver = "2"
members = ["game"]

[workspace.dependencies]
bevy = { version = "0.15", default-features = false, features = [
    "bevy_asset",
    "bevy_audio",
    "bevy_color",
    "bevy_core_pipeline",
    "bevy_gilrs",
    "bevy_gizmos",
    "bevy_gltf",
    "bevy_input_focus",
    "bevy_mesh_picking_backend",
    "bevy_pbr",
    "bevy_picking",
    "bevy_render",
    "bevy_scene",
    "bevy_sprite",
    "bevy_state",
    "bevy_text",
    "bevy_ui",
    "bevy_ui_picking_backend",
    "bevy_winit",
    "default_font",
    "hdr",
    "multi_threaded",
    "png",
    "smol_str",
    "sysinfo_plugin",
    "tonemapping_luts",
    "vorbis",
    "x11",
] }

# Optimize dependencies in dev builds for playable frame rates
[profile.dev.package."*"]
opt-level = 2

# Full optimization for release
[profile.release]
lto = "thin"
codegen-units = 1
```

In the game crate's `Cargo.toml`:

```toml
[package]
name = "my_game"
version = "0.1.0"
edition = "2021"

[dependencies]
bevy.workspace = true

[features]
dev = ["bevy/dynamic_linking"]
```

## Fast Compile Configuration

Create `.cargo/config.toml` to reduce compile times. See `references/cargo-config-templates.md` for platform-specific templates.

Key levers:

1. **Linker**: Use `mold` (Linux), `lld` (Windows), or the default macOS linker with proper flags.
2. **Dynamic linking**: Enable `bevy/dynamic_linking` during development via a `dev` feature flag. Run with `cargo run --features dev`.
3. **Cranelift backend** (optional, nightly): Faster codegen at the cost of runtime performance. Add to `.cargo/config.toml`:

```toml
# Requires: rustup component add rustc-codegen-cranelift --toolchain nightly
[unstable]
codegen-backend = true

[profile.dev]
codegen-backend = "cranelift"
```

## Minimal main.rs

A starter `main.rs` with window configuration:

```rust
use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "My Game".to_string(),
                resolution: (1280.0, 720.0).into(),
                ..default()
            }),
            ..default()
        }))
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
}
```

For a 3D starter, swap `Camera2d` for a 3D camera:

```rust
fn setup(mut commands: Commands) {
    // Camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(-2.5, 4.5, 9.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // Light
    commands.spawn((
        PointLight {
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0),
    ));
}
```

## Feature Flags

Bevy ships with many default features. For full-size games the defaults are fine. For specialized projects (headless server, minimal 2D game, CLI tool with ECS), disable defaults and pick only what you need.

See `references/feature-flags.md` for a complete table of flags with descriptions and guidance on when to enable or disable each one.

## WASM Target

To build for the web with Trunk:

1. Install prerequisites:

```bash
rustup target add wasm32-unknown-unknown
cargo install trunk
```

2. Create `index.html` in the project root:

```html
<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8" />
    <title>My Game</title>
    <style>
        html, body { margin: 0; padding: 0; width: 100%; height: 100%; overflow: hidden; }
        canvas { display: block; width: 100%; height: 100%; }
    </style>
</head>
<body>
    <link data-trunk rel="copy-dir" href="assets" />
</body>
</html>
```

3. Add a `wasm` feature in your game crate's `Cargo.toml` that selects only WASM-compatible Bevy features (no `x11`, `wayland`, `dynamic_linking`, `multi_threaded`):

```toml
[features]
wasm = [
    "bevy/bevy_asset",
    "bevy/bevy_audio",
    "bevy/bevy_color",
    "bevy/bevy_core_pipeline",
    "bevy/bevy_gizmos",
    "bevy/bevy_gltf",
    "bevy/bevy_input_focus",
    "bevy/bevy_pbr",
    "bevy/bevy_render",
    "bevy/bevy_scene",
    "bevy/bevy_sprite",
    "bevy/bevy_state",
    "bevy/bevy_text",
    "bevy/bevy_ui",
    "bevy/bevy_winit",
    "bevy/default_font",
    "bevy/hdr",
    "bevy/png",
    "bevy/tonemapping_luts",
    "bevy/vorbis",
    "bevy/webgl2",
]
```

4. Build and serve:

```bash
trunk serve --features wasm
```

Key WASM differences:
- Assets are loaded via HTTP, not the filesystem. Use `AssetServer` paths relative to the `assets/` directory.
- Audio requires a user interaction before it can play (browser policy).
- `bevy_gilrs` (gamepad support) does not work on WASM.
- Use `webgl2` for broad compatibility or `webgpu` for modern browsers only.
