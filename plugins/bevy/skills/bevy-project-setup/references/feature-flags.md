# Bevy Feature Flags Reference

Feature flags for Bevy 0.15. Disable `default-features` and pick what you need for minimal builds, or keep defaults for full-featured games.

## Core Features

| Feature | Default | Description | When to disable |
|---------|---------|-------------|-----------------|
| `multi_threaded` | Yes | Enables multi-threaded task execution | Headless single-threaded environments, WASM |
| `bevy_asset` | Yes | Asset loading system | Never for games; only for pure ECS/logic-only apps |
| `bevy_scene` | Yes | Scene serialization and loading | If you don't use `.scn.ron` scene files |
| `bevy_state` | Yes | State machine for app states (menu, gameplay, etc.) | Unlikely — most games need states |
| `bevy_color` | Yes | Color types and conversions | Rarely — almost everything uses colors |

## Windowing & Input

| Feature | Default | Description | When to disable |
|---------|---------|-------------|-----------------|
| `bevy_winit` | Yes | Window creation and event loop via winit | Headless/server builds |
| `bevy_gilrs` | Yes | Gamepad/controller support via gilrs | If you don't support gamepads; always disable for WASM |
| `bevy_input_focus` | Yes | Input focus tracking | Rarely |
| `bevy_picking` | Yes | Pointer-based picking (click/hover detection) | If you handle all input manually |
| `bevy_mesh_picking_backend` | Yes | Mesh-based picking for 3D objects | 2D-only games |
| `bevy_ui_picking_backend` | Yes | UI node picking | If not using bevy_ui |

## Rendering

| Feature | Default | Description | When to disable |
|---------|---------|-------------|-----------------|
| `bevy_render` | Yes | Core rendering infrastructure | Headless/server builds |
| `bevy_core_pipeline` | Yes | Built-in render pipelines (2D, 3D, tonemapping) | Headless/server builds |
| `bevy_pbr` | Yes | Physically-based 3D rendering, materials, lighting | 2D-only games |
| `bevy_sprite` | Yes | 2D sprite rendering | 3D-only games |
| `bevy_text` | Yes | Text rendering | If you never display text |
| `bevy_ui` | Yes | Built-in UI system | If using a third-party UI library exclusively |
| `bevy_gizmos` | Yes | Debug drawing (lines, shapes) | Production release builds (strip via feature) |
| `bevy_gltf` | Yes | glTF 3D model loading | 2D-only games or custom mesh generation |
| `hdr` | Yes | HDR texture support | If all textures are LDR |
| `tonemapping_luts` | Yes | Tonemapping look-up tables | If you use a custom tonemapper |

## Audio

| Feature | Default | Description | When to disable |
|---------|---------|-------------|-----------------|
| `bevy_audio` | Yes | Built-in audio playback | If using a third-party audio library (e.g., kira) |
| `vorbis` | Yes | OGG Vorbis audio decoding | If you only use WAV or other formats |

## Image Formats

| Feature | Default | Description | When to disable |
|---------|---------|-------------|-----------------|
| `png` | Yes | PNG image loading | If you only use other formats |
| `jpeg` | No | JPEG image loading | Enable if you have JPEG textures |
| `bmp` | No | BMP image loading | Enable if you have BMP textures |
| `ktx2` | No | KTX2 compressed texture loading | Enable for GPU-compressed textures |
| `basis-universal` | No | Basis Universal texture compression | Enable for cross-platform compressed textures |
| `exr` | No | OpenEXR HDR image loading | Enable for HDR environment maps |

## Platform Features

| Feature | Default | Description | When to disable |
|---------|---------|-------------|-----------------|
| `x11` | Yes | X11 windowing on Linux | Wayland-only Linux setups |
| `wayland` | No | Wayland windowing on Linux | Enable for native Wayland support |
| `webgl2` | No | WebGL2 rendering backend | Enable for WASM builds targeting broad browser support |
| `webgpu` | No | WebGPU rendering backend | Enable for WASM builds targeting modern browsers |

## Development & Debugging

| Feature | Default | Description | When to disable |
|---------|---------|-------------|-----------------|
| `dynamic_linking` | No | Dynamically link Bevy for faster dev compiles | Always disable for release/distribution builds |
| `file_watcher` | No | Hot-reload assets when files change on disk | Enable during development for asset iteration |
| `asset_processor` | No | Pre-process assets at build time | Enable when you need asset optimization pipelines |
| `embedded_watcher` | No | Hot-reload embedded assets | Enable during development with embedded assets |

## Profiling & Tracing

| Feature | Default | Description | When to disable |
|---------|---------|-------------|-----------------|
| `trace` | No | Adds tracing spans to Bevy systems and functions | Enable when profiling performance |
| `trace_tracy` | No | Tracy profiler integration | Enable to use the Tracy profiler |
| `trace_chrome` | No | Chrome trace format output (`chrome://tracing`) | Enable for browser-based trace viewing |
| `detailed_trace` | No | Verbose tracing for ECS internals | Enable only when debugging scheduler issues |

## Miscellaneous

| Feature | Default | Description | When to disable |
|---------|---------|-------------|-----------------|
| `default_font` | Yes | Bundles a default font so text works out of the box | If you always provide custom fonts |
| `smol_str` | Yes | Use `smol_str` for small-string optimization | Rarely needs disabling |
| `sysinfo_plugin` | Yes | System information diagnostics plugin | Production builds where you don't need diagnostics |
| `serialize` | No | Adds serde Serialize/Deserialize to common types | Enable for save/load systems or networking |
| `bevy_dev_tools` | No | Development tools (FPS overlay, state inspector) | Enable during development |

## Example: Minimal 2D Game

```toml
bevy = { version = "0.15", default-features = false, features = [
    "bevy_asset",
    "bevy_color",
    "bevy_core_pipeline",
    "bevy_render",
    "bevy_sprite",
    "bevy_state",
    "bevy_text",
    "bevy_ui",
    "bevy_winit",
    "default_font",
    "multi_threaded",
    "png",
    "x11",
] }
```

## Example: Headless Server

```toml
bevy = { version = "0.15", default-features = false, features = [
    "multi_threaded",
    "serialize",
] }
```
