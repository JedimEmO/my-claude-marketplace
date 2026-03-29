---
name: bevy-ui-and-audio
description: Use when the user asks about building UI in Bevy, game menus, HUD, health bars, buttons, text display, UI layout, or audio playback, sound effects, music, volume control, or spatial audio in Bevy.
version: 1.0.0
---

# Bevy UI & Audio — Game Interfaces & Sound

This skill covers two essential game systems: user interfaces (menus, HUD, buttons) and audio (music, SFX, spatial sound). Both rely on ECS fundamentals — see the `bevy-ecs` skill for components, systems, queries, and commands.

---

## UI with bevy_ui

Bevy ships a retained-mode UI system built on top of the Taffy layout engine (flexbox and CSS grid). UI elements are entities with `Node` and related components. They live in the ECS world alongside your game entities.

### Core UI Components

| Component | Purpose |
|-----------|---------|
| `Node` | Makes an entity a UI element. Carries all style/layout properties (width, height, flex direction, padding, etc.) |
| `Text` | Renders text. Requires a font `Handle<Font>` |
| `Button` | Marker that enables `Interaction` tracking on a `Node` |
| `ImageNode` | Displays an image inside a UI node |
| `BackgroundColor` | Solid color fill for a node |
| `BorderColor` | Border color (pair with `border` on `Node`) |
| `BorderRadius` | Rounded corners |
| `ZIndex` | Override draw order (`ZIndex::Local(i32)` or `ZIndex::Global(i32)`) |

### Layout Model

bevy_ui uses flexbox by default. Key style properties live directly on `Node`:

```rust
commands.spawn(Node {
    width: Val::Percent(100.0),
    height: Val::Px(60.0),
    flex_direction: FlexDirection::Row,
    justify_content: JustifyContent::SpaceBetween,
    align_items: AlignItems::Center,
    padding: UiRect::all(Val::Px(12.0)),
    column_gap: Val::Px(8.0),
    ..default()
});
```

**The `Val` enum** for sizing:
- `Val::Px(f32)` — absolute pixels
- `Val::Percent(f32)` — percentage of parent
- `Val::Auto` — automatic sizing (the default)
- `Val::Vw(f32)` / `Val::Vh(f32)` — viewport-relative

**Flexbox properties:**
- `flex_direction` — `Row`, `Column`, `RowReverse`, `ColumnReverse`
- `justify_content` — `Start`, `End`, `Center`, `SpaceBetween`, `SpaceAround`, `SpaceEvenly`
- `align_items` — `Start`, `End`, `Center`, `Stretch`, `Baseline`
- `align_self` — override parent's `align_items` for one child
- `flex_wrap` — `NoWrap`, `Wrap`, `WrapReverse`
- `flex_grow`, `flex_shrink`, `flex_basis` — standard flex sizing
- `row_gap`, `column_gap` — gap between children

**CSS Grid** is also supported:

```rust
commands.spawn(Node {
    display: Display::Grid,
    grid_template_columns: vec![
        GridTrack::flex(1.0),
        GridTrack::px(200.0),
        GridTrack::flex(2.0),
    ],
    grid_template_rows: vec![
        GridTrack::auto(),
        GridTrack::flex(1.0),
    ],
    ..default()
});
```

### Interaction and Buttons

The `Interaction` component is automatically added to entities with `Button`. Query it to detect clicks and hover:

```rust
fn button_system(
    mut query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, mut bg_color) in &mut query {
        match *interaction {
            Interaction::Pressed => {
                *bg_color = BackgroundColor(Color::srgb(0.35, 0.75, 0.35));
            }
            Interaction::Hovered => {
                *bg_color = BackgroundColor(Color::srgb(0.25, 0.25, 0.25));
            }
            Interaction::None => {
                *bg_color = BackgroundColor(Color::srgb(0.15, 0.15, 0.15));
            }
        }
    }
}
```

`Changed<Interaction>` is a query filter — the system only runs on entities whose `Interaction` actually changed this frame, avoiding unnecessary work.

**Focus policy**: By default, `Node` entities do not block interactions from reaching nodes behind them. Use `FocusPolicy::Block` to stop click-through:

```rust
commands.spawn((
    Node { ..default() },
    FocusPolicy::Block,
));
```

### UI Hierarchy — Nested Spawning

UI trees are built with `with_children`. The parent-child relationship drives layout (children are positioned inside parent nodes):

```rust
commands
    .spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        BackgroundColor(Color::NONE),
    ))
    .with_children(|parent| {
        parent
            .spawn((
                Button,
                Node {
                    width: Val::Px(200.0),
                    height: Val::Px(65.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    border: UiRect::all(Val::Px(2.0)),
                    ..default()
                },
                BorderColor(Color::WHITE),
                BackgroundColor(Color::srgb(0.15, 0.15, 0.15)),
            ))
            .with_children(|parent| {
                parent.spawn((
                    Text::new("Play"),
                    TextFont {
                        font_size: 28.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                ));
            });
    });
```

**TargetCamera**: To render UI on a specific camera (useful for split-screen or render-to-texture), add `TargetCamera(camera_entity)` to the root UI node.

### Common UI Patterns

**Main menu with navigation:**

```rust
#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
enum MenuState {
    #[default]
    Main,
    Settings,
    Credits,
}

#[derive(Component)]
enum MenuButton {
    Play,
    Settings,
    Quit,
}

fn spawn_main_menu(mut commands: Commands) {
    commands
        .spawn((
            StateScoped(MenuState::Main),
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                row_gap: Val::Px(16.0),
                ..default()
            },
        ))
        .with_children(|parent| {
            for (label, action) in [
                ("Play", MenuButton::Play),
                ("Settings", MenuButton::Settings),
                ("Quit", MenuButton::Quit),
            ] {
                parent
                    .spawn((
                        Button,
                        action,
                        Node {
                            width: Val::Px(250.0),
                            height: Val::Px(55.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.2, 0.2, 0.2)),
                    ))
                    .with_children(|btn| {
                        btn.spawn((
                            Text::new(label),
                            TextFont { font_size: 24.0, ..default() },
                            TextColor(Color::WHITE),
                        ));
                    });
            }
        });
}

fn handle_menu_buttons(
    query: Query<(&Interaction, &MenuButton), Changed<Interaction>>,
    mut next_state: ResMut<NextState<MenuState>>,
    mut exit: EventWriter<AppExit>,
) {
    for (interaction, button) in &query {
        if *interaction == Interaction::Pressed {
            match button {
                MenuButton::Play => { /* transition to GameState::Playing */ }
                MenuButton::Settings => next_state.set(MenuState::Settings),
                MenuButton::Quit => { exit.write(AppExit::Success); }
            }
        }
    }
}
```

`StateScoped` despawns the entity (and its children) automatically when leaving that state — no manual cleanup needed.

**HUD overlay (health bar + score):**

```rust
#[derive(Component)]
struct HealthBar;

#[derive(Component)]
struct ScoreText;

fn spawn_hud(mut commands: Commands) {
    // Root container pinned to top of screen
    commands
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Px(40.0),
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Center,
            padding: UiRect::horizontal(Val::Px(16.0)),
            ..default()
        })
        .with_children(|parent| {
            // Health bar: background + fill
            parent
                .spawn(Node {
                    width: Val::Px(200.0),
                    height: Val::Px(20.0),
                    ..default()
                })
                .insert(BackgroundColor(Color::srgb(0.3, 0.0, 0.0)))
                .with_children(|bar_bg| {
                    bar_bg.spawn((
                        HealthBar,
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.0, 0.8, 0.0)),
                    ));
                });

            // Score text
            parent.spawn((
                ScoreText,
                Text::new("Score: 0"),
                TextFont { font_size: 20.0, ..default() },
                TextColor(Color::WHITE),
            ));
        });
}

fn update_health_bar(
    player: Query<&Health, With<Player>>,
    mut bar: Query<&mut Node, With<HealthBar>>,
) {
    if let (Ok(health), Ok(mut node)) = (player.single(), bar.single_mut()) {
        node.width = Val::Percent(health.current as f32 / health.max as f32 * 100.0);
    }
}
```

**Loading screen with progress bar:**

```rust
#[derive(Resource, Default)]
struct LoadingProgress {
    loaded: usize,
    total: usize,
}

fn update_loading_bar(
    progress: Res<LoadingProgress>,
    mut bar: Query<&mut Node, With<LoadingBar>>,
) {
    if let Ok(mut node) = bar.single_mut() {
        let pct = if progress.total > 0 {
            progress.loaded as f32 / progress.total as f32 * 100.0
        } else {
            0.0
        };
        node.width = Val::Percent(pct);
    }
}
```

### bevy_egui — Debug and Editor UI

Use `bevy_egui` for debug panels, inspector tools, and editor UI. Use `bevy_ui` for in-game UI that ships to players.

**When to choose bevy_egui:**
- Rapid prototyping — egui is immediate-mode, faster to iterate
- Debug overlays, entity inspectors, level editors
- You need text input fields, sliders, collapsible panels, drag-and-drop

**When to choose bevy_ui:**
- Final in-game UI (menus, HUD, dialogue boxes)
- You need pixel-perfect control, custom rendering, animations
- Performance-sensitive UI (bevy_ui is integrated with the render pipeline)

**Setup:**

```toml
# Cargo.toml
[dependencies]
bevy_egui = "0.34"  # Match your Bevy version
```

```rust
use bevy_egui::{egui, EguiContexts, EguiPlugin};

app.add_plugins(EguiPlugin);

fn debug_ui(mut contexts: EguiContexts) {
    egui::Window::new("Debug").show(contexts.ctx_mut(), |ui| {
        ui.label("Hello from egui");
        if ui.button("Click me").clicked() {
            // handle click
        }
    });
}
```

---

## Audio

Bevy's built-in audio supports loading sound files, playing one-shot effects, looping music, volume control, and spatial 3D audio.

### Audio Basics

Audio in Bevy works through entities. You load audio files as assets, then spawn entities with an `AudioPlayer` component to play them:

```rust
// Load audio assets (typically in a setup system)
fn setup_audio(mut commands: Commands, asset_server: Res<AssetServer>) {
    let music_handle: Handle<AudioSource> = asset_server.load("audio/background.ogg");
    let sfx_handle: Handle<AudioSource> = asset_server.load("audio/explosion.ogg");

    // Store handles in a resource for later use
    commands.insert_resource(GameAudio {
        music: music_handle,
        explosion: sfx_handle,
    });
}

#[derive(Resource)]
struct GameAudio {
    music: Handle<AudioSource>,
    explosion: Handle<AudioSource>,
}
```

Supported formats: OGG Vorbis, WAV, FLAC, MP3 (via feature flags — `ogg` is enabled by default).

### Playing Sounds

**Background music (looping):**

```rust
fn start_music(mut commands: Commands, audio: Res<GameAudio>) {
    commands.spawn((
        AudioPlayer(audio.music.clone()),
        PlaybackSettings::LOOP,
    ));
}
```

**One-shot SFX:**

```rust
fn play_explosion(mut commands: Commands, audio: Res<GameAudio>) {
    commands.spawn((
        AudioPlayer(audio.explosion.clone()),
        PlaybackSettings::DESPAWN, // Entity is despawned when playback finishes
    ));
}
```

**PlaybackSettings presets:**

| Preset | Behavior |
|--------|----------|
| `PlaybackSettings::ONCE` | Play once, entity remains after completion |
| `PlaybackSettings::LOOP` | Loop forever |
| `PlaybackSettings::DESPAWN` | Play once, despawn entity on finish |
| `PlaybackSettings::REMOVE` | Play once, remove audio components on finish (entity stays) |

**Custom settings:**

```rust
PlaybackSettings {
    mode: PlaybackMode::Loop,
    volume: Volume::new(0.5),
    speed: 1.2,
    paused: false,
    spatial: false,
    spatial_scale: None,
}
```

### Controlling Playback

Once an audio entity is playing, Bevy adds an `AudioSink` component to it. Query this to control playback at runtime:

```rust
#[derive(Component)]
struct MusicTrack;

// Spawn tagged music
fn start_music(mut commands: Commands, audio: Res<GameAudio>) {
    commands.spawn((
        MusicTrack,
        AudioPlayer(audio.music.clone()),
        PlaybackSettings::LOOP,
    ));
}

// Pause/resume
fn toggle_music(
    query: Query<&AudioSink, With<MusicTrack>>,
    input: Res<ButtonInput<KeyCode>>,
) {
    if input.just_pressed(KeyCode::KeyM) {
        if let Ok(sink) = query.single() {
            sink.toggle();  // pause if playing, resume if paused
        }
    }
}

// Adjust volume
fn set_volume(query: Query<&AudioSink, With<MusicTrack>>) {
    if let Ok(sink) = query.single() {
        sink.set_volume(0.3);  // 0.0 = silent, 1.0 = full
    }
}

// Stop and remove
fn stop_music(
    mut commands: Commands,
    query: Query<(Entity, &AudioSink), With<MusicTrack>>,
) {
    if let Ok((entity, sink)) = query.single() {
        sink.stop();
        commands.entity(entity).despawn();
    }
}
```

**AudioSink methods:**
- `toggle()` — pause/resume
- `pause()`, `play()` — explicit pause/resume
- `stop()` — stop playback
- `set_volume(f32)` — set volume (0.0 to 1.0+)
- `set_speed(f32)` — set playback speed
- `is_paused() -> bool`
- `empty() -> bool` — true when playback finished

### Spatial Audio

Spatial audio positions sounds in 3D space. Sounds get louder or quieter based on the listener's distance, and pan left/right based on direction.

**Setup a spatial listener:**

```rust
fn setup_spatial(mut commands: Commands) {
    // The listener is typically on the player or camera
    commands.spawn((
        Transform::default(),
        SpatialListener::default(),
        // Usually bundled with your camera or player entity
    ));
}
```

**Spawn a spatial sound source:**

```rust
fn spawn_ambient_sound(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        AudioPlayer(asset_server.load("audio/campfire.ogg")),
        PlaybackSettings {
            mode: PlaybackMode::Loop,
            spatial: true,
            ..default()
        },
        Transform::from_xyz(10.0, 0.0, -5.0),
    ));
}
```

The sound's `Transform` position relative to the `SpatialListener`'s position determines volume and panning. As the listener (player/camera) moves closer, the sound gets louder.

**Spatial scale**: Control how quickly sounds attenuate with distance using `SpatialScale` as a resource:

```rust
app.insert_resource(SpatialScale::new(1.0)); // Default; smaller = slower falloff
```

### bevy_kira_audio — Advanced Audio

For games that need crossfading, audio channels, streaming, or fine-grained audio control, `bevy_kira_audio` wraps the Kira audio library:

**When to use bevy_kira_audio over built-in audio:**
- Crossfading between music tracks
- Named audio channels (music, SFX, ambient, voice) with independent volume
- Audio tweening (fade in/out over duration)
- Streaming large audio files
- Audio instances with per-instance control

**Setup:**

```toml
# Cargo.toml — replace default bevy audio
[dependencies]
bevy = { version = "0.15", default-features = false, features = [
    # include your needed features, but NOT bevy_audio
] }
bevy_kira_audio = "0.22"  # Match your Bevy version
```

```rust
use bevy_kira_audio::prelude::*;

app.add_plugins(AudioPlugin);  // bevy_kira_audio's AudioPlugin

// Play with channels and fading
fn play_music(audio: Res<Audio>, asset_server: Res<AssetServer>) {
    audio
        .play(asset_server.load("audio/music.ogg"))
        .looped()
        .with_volume(0.7)
        .fade_in(AudioTween::linear(Duration::from_secs(2)));
}
```

**Audio channels** for independent volume control:

```rust
#[derive(Resource)]
struct MusicChannel;

#[derive(Resource)]
struct SfxChannel;

app.add_audio_channel::<MusicChannel>()
   .add_audio_channel::<SfxChannel>();

fn adjust_music_volume(channel: Res<AudioChannel<MusicChannel>>) {
    channel.set_volume(0.5);
}
```

Choose built-in Bevy audio for simple games and prototypes. Reach for `bevy_kira_audio` when you need production audio features like crossfading and channel mixing.
