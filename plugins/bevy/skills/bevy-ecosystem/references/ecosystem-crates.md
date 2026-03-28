# Ecosystem Crates Reference

Detailed reference for recommended third-party Bevy crates. All versions listed target **Bevy 0.15.x**. Always verify compatibility before adding a dependency.

---

## Physics

### avian3d / avian2d

ECS-native physics engine designed specifically for Bevy. Successor to bevy_xpbd. Preferred for new projects.

```toml
# Cargo.toml
avian3d = "0.2"
# or for 2D:
avian2d = "0.2"
```

```rust
app.add_plugins(avian3d::PhysicsPlugins::default());
```

- Repo: https://github.com/Jondolf/avian
- Docs: https://docs.rs/avian3d

### bevy_rapier3d / bevy_rapier2d

Rapier physics engine integration. Mature and battle-tested.

```toml
bevy_rapier3d = "0.28"
# or for 2D:
bevy_rapier2d = "0.28"
```

```rust
app.add_plugins(RapierPhysicsPlugin::<NoUserData>::default());
```

- Repo: https://github.com/dimforge/bevy_rapier
- Docs: https://docs.rs/bevy_rapier3d

---

## Input

### leafwing-input-manager

Declarative input mapping: bind actions to keys, buttons, gamepads, mouse, or virtual axes. Supports combos, chords, and input contexts.

```toml
leafwing-input-manager = "0.16"
```

```rust
app.add_plugins(InputManagerPlugin::<MyAction>::default());
```

- Repo: https://github.com/Leafwing-Studios/leafwing-input-manager
- Docs: https://docs.rs/leafwing-input-manager

---

## Assets & Loading

### bevy_asset_loader

Declarative asset loading — define asset collections with derive macros, load them during a loading state, get notified when complete.

```toml
bevy_asset_loader = "0.22"
```

```rust
app.add_plugins(AssetLoaderPlugin::new(GameState::Loading, GameState::Playing));
```

- Repo: https://github.com/NiklasEi/bevy_asset_loader
- Docs: https://docs.rs/bevy_asset_loader

### iyes_progress

Track loading progress across multiple systems. Pairs well with bevy_asset_loader.

```toml
iyes_progress = "0.13"
```

```rust
app.add_plugins(ProgressPlugin::<GameState>::new().with_state(GameState::Loading));
```

- Repo: https://github.com/IyesGames/iyes_progress
- Docs: https://docs.rs/iyes_progress

---

## Animation

### bevy_tweening

Component and resource tweens — animate transforms, colors, and custom lenses over time with easing functions and sequences.

```toml
bevy_tweening = "0.12"
```

```rust
app.add_plugins(TweeningPlugin);
```

- Repo: https://github.com/djeedai/bevy_tweening
- Docs: https://docs.rs/bevy_tweening

---

## UI & Editor

### bevy_egui

Immediate-mode egui rendered inside Bevy. Ideal for debug panels, level editors, and dev tools. Not recommended for in-game UI.

```toml
bevy_egui = "0.34"
```

```rust
app.add_plugins(EguiPlugin);
```

- Repo: https://github.com/mvlabat/bevy_egui
- Docs: https://docs.rs/bevy_egui

### bevy_cosmic_edit

Rich text editing widget using cosmic-text. Supports multi-line editing, selection, clipboard, and custom fonts.

```toml
bevy_cosmic_edit = "0.27"
```

```rust
app.add_plugins(CosmicEditPlugin::default());
```

- Repo: https://github.com/StaffEngineer/bevy_cosmic_edit
- Docs: https://docs.rs/bevy_cosmic_edit

---

## Networking

### lightyear

Client-prediction and server-authoritative networking. Supports rollback, input delay, entity interpolation, and interest management.

```toml
lightyear = "0.19"
```

```rust
app.add_plugins(lightyear::prelude::server::ServerPlugins::default());
// or
app.add_plugins(lightyear::prelude::client::ClientPlugins::default());
```

- Repo: https://github.com/cBournhonesque/lightyear
- Docs: https://docs.rs/lightyear

### bevy_replicon

High-level replication: automatic entity spawning on clients, component synchronization, and server RPCs. Transport-agnostic.

```toml
bevy_replicon = "0.30"
```

```rust
app.add_plugins(RepliconPlugins);
```

- Repo: https://github.com/projectharmonia/bevy_replicon
- Docs: https://docs.rs/bevy_replicon

### bevy_renet

Reliable UDP transport for Bevy. Lower-level than lightyear or replicon — gives you raw channels and connection management.

```toml
bevy_renet = "0.0.14"
```

```rust
app.add_plugins(RenetServerPlugin);
// or
app.add_plugins(RenetClientPlugin);
```

- Repo: https://github.com/lucaspoffo/renet
- Docs: https://docs.rs/bevy_renet

---

## Debug & Dev Tools

### bevy-inspector-egui

Runtime ECS inspector. Browse all entities, view and edit component values live, inspect resources. Essential during development.

```toml
bevy-inspector-egui = "0.28"
```

```rust
app.add_plugins(bevy_inspector_egui::quick::WorldInspectorPlugin::default());
```

- Repo: https://github.com/jakobhellermann/bevy-inspector-egui
- Docs: https://docs.rs/bevy-inspector-egui

### bevy_screen_diagnostics

On-screen text overlay showing FPS, entity count, and custom diagnostics. Lightweight, no egui dependency.

```toml
bevy_screen_diagnostics = "0.7"
```

```rust
app.add_plugins(ScreenDiagnosticsPlugin::default())
   .add_plugins(ScreenFrameDiagnosticsPlugin);
```

- Repo: https://github.com/Lommix/bevy_screen_diagnostics
- Docs: https://docs.rs/bevy_screen_diagnostics

---

## Tilemap & Level Design

### bevy_ecs_tilemap

High-performance tilemap rendering backed by the ECS. Supports multiple layers, animated tiles, and large maps.

```toml
bevy_ecs_tilemap = "0.15"
```

```rust
app.add_plugins(TilemapPlugin);
```

- Repo: https://github.com/StarArawn/bevy_ecs_tilemap
- Docs: https://docs.rs/bevy_ecs_tilemap

---

## Particles & VFX

### bevy_hanabi

GPU-accelerated particle system. Define effects with spawners, modifiers, and render properties. Handles millions of particles.

```toml
bevy_hanabi = "0.14"
```

```rust
app.add_plugins(HanabiPlugin);
```

- Repo: https://github.com/djeedai/bevy_hanabi
- Docs: https://docs.rs/bevy_hanabi

---

## Camera

### bevy_pancam

Plug-and-play 2D camera with pan (drag), zoom (scroll), and optional bounds clamping.

```toml
bevy_pancam = "0.14"
```

```rust
app.add_plugins(PanCamPlugin);
// Then add PanCam component to your camera entity
```

- Repo: https://github.com/johanhelsing/bevy_pancam
- Docs: https://docs.rs/bevy_pancam

### bevy_flycam

Simple 3D fly camera for prototyping. WASD + mouse look, adjustable speed.

```toml
bevy_flycam = "0.14"
```

```rust
app.add_plugins(NoCameraPlayerPlugin);
// Spawns and controls a camera automatically
```

- Repo: https://github.com/sburris0/bevy_flycam
- Docs: https://docs.rs/bevy_flycam

---

## Persistence & Serialization

### bevy_pkv

Simple key-value store for game settings and save data. Uses sled on native and localStorage on WASM.

```toml
bevy_pkv = "0.12"
```

```rust
app.add_plugins(PkvPlugin::new("MyCompany", "MyGame"));
```

```rust
// Writing
fn save_settings(mut pkv: ResMut<PkvStore>) {
    pkv.set("volume", &0.8f32).expect("failed to save");
}

// Reading
fn load_settings(pkv: Res<PkvStore>) {
    let volume: f32 = pkv.get("volume").unwrap_or(1.0);
}
```

- Repo: https://github.com/johanhelsing/bevy_pkv
- Docs: https://docs.rs/bevy_pkv

---

## Audio (Third-Party)

### bevy_kira_audio

Advanced audio playback powered by the Kira audio library. Supports spatial audio, audio tweening, multiple channels, and precise timing.

```toml
bevy_kira_audio = "0.21"
```

```rust
app.add_plugins(AudioPlugin);
```

```rust
fn play_bgm(audio: Res<Audio>, assets: Res<AssetServer>) {
    audio.play(assets.load("bgm.ogg")).looped().with_volume(0.5);
}
```

- Repo: https://github.com/NiklasEi/bevy_kira_audio
- Docs: https://docs.rs/bevy_kira_audio
