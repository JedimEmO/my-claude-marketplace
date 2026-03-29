---
name: bevy-rendering
description: Use when the user asks about 2D or 3D rendering in Bevy, sprites, meshes, materials, cameras, lighting, shaders, textures, transforms, visibility, render layers, viewports, or visual aspects of a Bevy game.
version: 1.0.0
---

# Bevy Rendering — 2D & 3D Visuals

This skill covers Bevy's rendering systems for both 2D and 3D. For component and system fundamentals, see the `bevy-ecs` skill.

All examples target **Bevy 0.15+** APIs, which use individual components rather than the deprecated bundle pattern.

---

## Transform Hierarchy

Every visible entity needs a `Transform` (local) and `GlobalTransform` (computed world-space). Bevy propagates transforms through parent-child relationships automatically.

**Coordinate system**: right-handed, Y-up. +X is right, +Y is up, +Z points toward the viewer.

```rust
use bevy::prelude::*;

fn setup(mut commands: Commands) {
    // Parent entity
    let parent = commands.spawn((
        Transform::from_xyz(0.0, 2.0, 0.0),
        Visibility::default(),
    )).id();

    // Child — its Transform is relative to the parent
    commands.spawn((
        Transform::from_xyz(1.0, 0.0, 0.0), // world position: (1.0, 2.0, 0.0)
        Visibility::default(),
    )).set_parent(parent);
}
```

Key transform methods:

```rust
Transform::from_xyz(x, y, z)
Transform::from_translation(Vec3::new(x, y, z))
Transform::from_rotation(Quat::from_rotation_y(angle))
Transform::from_scale(Vec3::splat(2.0))
transform.looking_at(target, Vec3::Y)  // orient to face a point
```

---

## 2D Rendering

### Sprites

```rust
fn setup_sprite(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2d);

    commands.spawn((
        Sprite {
            image: asset_server.load("player.png"),
            color: Color::WHITE,
            custom_size: Some(Vec2::new(64.0, 64.0)),  // optional override
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));
}
```

### Z-ordering in 2D

Use `transform.translation.z` to control draw order. Higher Z values render on top.

```rust
// Background at z=0, player at z=1, UI overlay at z=2
commands.spawn((
    Sprite { image: asset_server.load("bg.png"), ..default() },
    Transform::from_xyz(0.0, 0.0, 0.0),
));
commands.spawn((
    Sprite { image: asset_server.load("player.png"), ..default() },
    Transform::from_xyz(0.0, 0.0, 1.0),
));
```

### Sprite Sheets with TextureAtlas

```rust
fn setup_spritesheet(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let texture = asset_server.load("spritesheet.png");
    let layout = TextureAtlasLayout::from_grid(UVec2::new(32, 32), 6, 1, None, None);
    let layout_handle = texture_atlas_layouts.add(layout);

    commands.spawn((
        Sprite {
            image: texture,
            texture_atlas: Some(TextureAtlas {
                layout: layout_handle,
                index: 0,
            }),
            ..default()
        },
        Transform::default(),
    ));
}
```

### Sprite Animation

```rust
#[derive(Component)]
struct AnimationTimer(Timer);

fn animate_sprite(
    time: Res<Time>,
    mut query: Query<(&mut AnimationTimer, &mut Sprite)>,
) {
    for (mut timer, mut sprite) in &mut query {
        timer.0.tick(time.delta());
        if timer.0.just_finished() {
            if let Some(atlas) = &mut sprite.texture_atlas {
                atlas.index = (atlas.index + 1) % 6; // 6 frames
            }
        }
    }
}
```

### Camera2d

```rust
commands.spawn((
    Camera2d,
    Transform::from_xyz(0.0, 0.0, 0.0),
    OrthographicProjection {
        scale: 1.0, // zoom: smaller = zoomed in
        ..OrthographicProjection::default_2d()
    },
));
```

---

## 3D Rendering

### Meshes and Materials

Bevy uses a PBR (Physically Based Rendering) pipeline. Attach `Mesh3d` and `MeshMaterial3d<StandardMaterial>` components.

```rust
fn setup_3d(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Spawn a red cube
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.8, 0.1, 0.1),
            metallic: 0.0,
            perceptual_roughness: 0.5,
            ..default()
        })),
        Transform::from_xyz(0.0, 0.5, 0.0),
    ));
}
```

### StandardMaterial Properties

| Property | Type | Description |
|---|---|---|
| `base_color` | `Color` | Albedo color |
| `base_color_texture` | `Option<Handle<Image>>` | Albedo texture map |
| `metallic` | `f32` | 0.0 = dielectric, 1.0 = metal |
| `perceptual_roughness` | `f32` | 0.0 = mirror-smooth, 1.0 = rough |
| `emissive` | `LinearRgba` | Self-illumination color (not affected by lighting) |
| `reflectance` | `f32` | Fresnel reflectance at normal incidence (default 0.5) |
| `alpha_mode` | `AlphaMode` | Opaque, Blend, Mask, etc. |
| `double_sided` | `bool` | Render back faces |
| `unlit` | `bool` | Skip lighting calculations |

### Built-in Shape Primitives

All implement `Into<Mesh>`:

```rust
Cuboid::new(width, height, depth)
Sphere::new(radius).mesh().ico(subdivisions)   // or .uv(sectors, stacks)
Plane3d::default().mesh().size(width, depth)
Cylinder::new(radius, height)
Capsule3d::new(radius, half_length)
Torus::new(inner_radius, outer_radius)
```

---

## Cameras

### Camera3d

```rust
commands.spawn((
    Camera3d::default(),
    Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
));
```

### Orthographic vs Perspective

```rust
// Perspective (default for Camera3d)
commands.spawn((
    Camera3d::default(),
    Projection::Perspective(PerspectiveProjection {
        fov: std::f32::consts::FRAC_PI_4,
        ..default()
    }),
    Transform::from_xyz(0.0, 5.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
));

// Orthographic 3D
commands.spawn((
    Camera3d::default(),
    Projection::Orthographic(OrthographicProjection {
        scale: 10.0,
        ..OrthographicProjection::default_3d()
    }),
    Transform::from_xyz(0.0, 5.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
));
```

### Multi-Camera Setup

Use `order` to control rendering order and `ClearColorConfig` to avoid clearing previous camera output.

```rust
// Primary camera — renders first, clears to sky blue
commands.spawn((
    Camera3d::default(),
    Camera {
        order: 0,
        clear_color: ClearColorConfig::Custom(Color::srgb(0.5, 0.7, 1.0)),
        ..default()
    },
    Transform::from_xyz(0.0, 5.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
));

// Secondary camera — renders on top, does not clear
commands.spawn((
    Camera3d::default(),
    Camera {
        order: 1,
        clear_color: ClearColorConfig::None,
        ..default()
    },
    Transform::from_xyz(10.0, 5.0, 0.0).looking_at(Vec3::ZERO, Vec3::Y),
));
```

### Viewports

```rust
use bevy::render::camera::Viewport;

commands.spawn((
    Camera3d::default(),
    Camera {
        viewport: Some(Viewport {
            physical_position: UVec2::new(0, 0),
            physical_size: UVec2::new(640, 480),
            ..default()
        }),
        ..default()
    },
    Transform::from_xyz(0.0, 5.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
));
```

---

## Lighting

### Light Types

```rust
fn setup_lights(mut commands: Commands) {
    // Directional light (sun-like, infinite distance)
    commands.spawn((
        DirectionalLight {
            illuminance: 10_000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::default().looking_at(Vec3::new(-1.0, -1.0, -1.0), Vec3::Y),
    ));

    // Point light (omni-directional, positioned in space)
    commands.spawn((
        PointLight {
            color: Color::srgb(1.0, 0.9, 0.8),
            intensity: 1_000_000.0, // lumens
            range: 20.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0),
    ));

    // Spot light (cone-shaped)
    commands.spawn((
        SpotLight {
            color: Color::WHITE,
            intensity: 1_000_000.0,
            range: 30.0,
            outer_angle: std::f32::consts::FRAC_PI_4,
            inner_angle: std::f32::consts::FRAC_PI_6,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(0.0, 10.0, 0.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // Ambient light (uniform, no direction or position)
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 100.0,
    });
}
```

### Shadow Configuration

Shadows are enabled per-light with `shadows_enabled: true`. For directional lights, configure the shadow cascade:

```rust
commands.spawn((
    DirectionalLight {
        shadows_enabled: true,
        ..default()
    },
    CascadeShadowConfig::build(CascadeShadowConfigBuilder {
        num_cascades: 4,
        maximum_distance: 100.0,
        first_cascade_far_bound: 5.0,
        ..default()
    }),
    Transform::default().looking_at(Vec3::new(-1.0, -1.0, -1.0), Vec3::Y),
));
```

---

## Asset Loading

### AssetServer Basics

```rust
fn load_assets(asset_server: Res<AssetServer>) {
    // Loads from `assets/` directory relative to the project root
    let texture: Handle<Image> = asset_server.load("textures/wall.png");
    let font: Handle<Font> = asset_server.load("fonts/FiraSans-Bold.ttf");
    let scene: Handle<Scene> = asset_server.load("models/character.glb#Scene0");
}
```

### GLTF / GLB Models

Use `SceneRoot` to spawn an entire GLTF scene:

```rust
fn load_model(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        SceneRoot(asset_server.load("models/helmet.glb#Scene0")),
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));
}
```

For specific meshes or materials from a GLTF file:

```rust
// Load a specific named mesh
let mesh: Handle<Mesh> = asset_server.load("models/character.glb#Mesh0/Primitive0");
```

### Asset Load State

```rust
fn check_loading(
    asset_server: Res<AssetServer>,
    texture: Res<MyTextureHandle>,  // store handle in a resource
) {
    match asset_server.get_load_state(&texture.0) {
        Some(bevy::asset::LoadState::Loaded) => { /* ready to use */ }
        Some(bevy::asset::LoadState::Failed(_)) => { /* handle error */ }
        _ => { /* still loading */ }
    }
}
```

---

## Text Rendering

### 2D Text

```rust
fn setup_text(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font = asset_server.load("fonts/FiraSans-Bold.ttf");

    commands.spawn((
        Text2d::new("Hello, Bevy!"),
        TextFont {
            font: font.clone(),
            font_size: 48.0,
            ..default()
        },
        TextColor(Color::WHITE),
        TextLayout::new_with_justify(JustifyText::Center),
        Transform::from_xyz(0.0, 0.0, 10.0),
    ));
}
```

---

## Visibility

### Visibility Component

Every rendered entity has a `Visibility` component controlling whether it is drawn:

```rust
// Visible — always rendered (overrides parent hidden)
commands.spawn((
    Sprite { image: asset_server.load("icon.png"), ..default() },
    Visibility::Visible,
    Transform::default(),
));

// Hidden — never rendered (children also hidden)
commands.spawn((
    Sprite { image: asset_server.load("icon.png"), ..default() },
    Visibility::Hidden,
    Transform::default(),
));

// Inherited (default) — visible if parent is visible
commands.spawn((
    Sprite { image: asset_server.load("icon.png"), ..default() },
    Visibility::default(), // Inherited
    Transform::default(),
));
```

Toggle visibility at runtime:

```rust
fn toggle_visibility(mut query: Query<&mut Visibility, With<MyMarker>>) {
    for mut vis in &mut query {
        *vis = match *vis {
            Visibility::Hidden => Visibility::Visible,
            _ => Visibility::Hidden,
        };
    }
}
```

### InheritedVisibility

`InheritedVisibility` is a read-only computed component. It reflects the effective visibility considering the entire parent chain. Use it to check whether an entity is actually visible on screen.

### RenderLayers

Use `RenderLayers` to control which camera sees which entities. Both the camera and the entity must share at least one layer.

```rust
use bevy::render::view::RenderLayers;

// Entity on layer 1 only
commands.spawn((
    Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
    MeshMaterial3d(materials.add(Color::srgb(0.8, 0.2, 0.2))),
    Transform::default(),
    RenderLayers::layer(1),
));

// Camera that sees layers 0 and 1
commands.spawn((
    Camera3d::default(),
    Transform::from_xyz(0.0, 5.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
    RenderLayers::from_layers(&[0, 1]),
));
```
