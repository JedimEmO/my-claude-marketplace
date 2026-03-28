# Common Rendering Recipes

Minimal, copy-pasteable examples for frequent Bevy 0.15+ rendering tasks.

---

## 1. Animated Sprite Sheet

Load a sprite sheet atlas and cycle through frames with a timer.

```rust
use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest())) // pixel-art friendly
        .add_systems(Startup, setup)
        .add_systems(Update, animate_sprite)
        .run();
}

#[derive(Component)]
struct AnimationConfig {
    first_frame: usize,
    last_frame: usize,
    timer: Timer,
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    commands.spawn(Camera2d);

    let texture = asset_server.load("characters/player_run.png");
    // 6 frames in a horizontal strip, each 32x32 pixels
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
        Transform::from_scale(Vec3::splat(4.0)), // scale up for visibility
        AnimationConfig {
            first_frame: 0,
            last_frame: 5,
            timer: Timer::from_seconds(0.1, TimerMode::Repeating),
        },
    ));
}

fn animate_sprite(
    time: Res<Time>,
    mut query: Query<(&mut AnimationConfig, &mut Sprite)>,
) {
    for (mut config, mut sprite) in &mut query {
        config.timer.tick(time.delta());
        if config.timer.just_finished() {
            if let Some(atlas) = &mut sprite.texture_atlas {
                atlas.index = if atlas.index >= config.last_frame {
                    config.first_frame
                } else {
                    atlas.index + 1
                };
            }
        }
    }
}
```

---

## 2. 3D Scene with Lighting

Ground plane, a lit object, directional light, and ambient light.

```rust
use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Ground plane
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(10.0, 10.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.3, 0.5, 0.3),
            perceptual_roughness: 0.9,
            ..default()
        })),
        Transform::default(),
    ));

    // Lit cube
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.8, 0.2, 0.2),
            metallic: 0.3,
            perceptual_roughness: 0.4,
            ..default()
        })),
        Transform::from_xyz(0.0, 0.5, 0.0),
    ));

    // Directional light (sun)
    commands.spawn((
        DirectionalLight {
            illuminance: 15_000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::default().looking_at(Vec3::new(-1.0, -2.0, -1.5), Vec3::Y),
    ));

    // Ambient fill
    commands.insert_resource(AmbientLight {
        color: Color::srgb(0.6, 0.7, 1.0),
        brightness: 200.0,
    });

    // Camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(-3.0, 3.0, 5.0).looking_at(Vec3::new(0.0, 0.5, 0.0), Vec3::Y),
    ));
}
```

---

## 3. Split-Screen Two-Camera Setup

Left half shows one camera, right half shows another.

```rust
use bevy::{prelude::*, render::camera::Viewport};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, update_viewports)
        .run();
}

#[derive(Component)]
struct LeftCamera;

#[derive(Component)]
struct RightCamera;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Shared scene content
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
        MeshMaterial3d(materials.add(Color::srgb(0.8, 0.2, 0.2))),
        Transform::from_xyz(0.0, 0.5, 0.0),
    ));
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(10.0, 10.0))),
        MeshMaterial3d(materials.add(Color::srgb(0.3, 0.5, 0.3))),
        Transform::default(),
    ));
    commands.spawn((
        DirectionalLight {
            shadows_enabled: true,
            ..default()
        },
        Transform::default().looking_at(Vec3::new(-1.0, -1.0, -1.0), Vec3::Y),
    ));

    // Left camera — renders first, clears background
    commands.spawn((
        Camera3d::default(),
        Camera {
            order: 0,
            clear_color: ClearColorConfig::Custom(Color::srgb(0.1, 0.1, 0.2)),
            ..default()
        },
        Transform::from_xyz(-3.0, 3.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        LeftCamera,
    ));

    // Right camera — renders second, does not clear the left half
    commands.spawn((
        Camera3d::default(),
        Camera {
            order: 1,
            clear_color: ClearColorConfig::None,
            ..default()
        },
        Transform::from_xyz(5.0, 3.0, -3.0).looking_at(Vec3::ZERO, Vec3::Y),
        RightCamera,
    ));
}

fn update_viewports(
    windows: Query<&Window>,
    mut left_camera: Query<&mut Camera, (With<LeftCamera>, Without<RightCamera>)>,
    mut right_camera: Query<&mut Camera, (With<RightCamera>, Without<LeftCamera>)>,
) {
    let Ok(window) = windows.single() else { return };
    let width = window.physical_width();
    let height = window.physical_height();
    let half_width = width / 2;

    if let Ok(mut cam) = left_camera.single_mut() {
        cam.viewport = Some(Viewport {
            physical_position: UVec2::ZERO,
            physical_size: UVec2::new(half_width, height),
            ..default()
        });
    }

    if let Ok(mut cam) = right_camera.single_mut() {
        cam.viewport = Some(Viewport {
            physical_position: UVec2::new(half_width, 0),
            physical_size: UVec2::new(width - half_width, height),
            ..default()
        });
    }
}
```

---

## 4. Loading and Displaying a GLTF Model

Load a `.glb` file and spawn its scene.

```rust
use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Load the GLTF scene (Scene0 is the first/default scene)
    commands.spawn((
        SceneRoot(asset_server.load("models/FlightHelmet.glb#Scene0")),
        Transform::from_xyz(0.0, 0.0, 0.0)
            .with_scale(Vec3::splat(3.0)),
    ));

    // Lighting
    commands.spawn((
        DirectionalLight {
            illuminance: 20_000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::default().looking_at(Vec3::new(-1.0, -1.0, -1.0), Vec3::Y),
    ));
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 300.0,
    });

    // Camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 1.5, 4.0).looking_at(Vec3::new(0.0, 0.8, 0.0), Vec3::Y),
    ));
}
```

---

## 5. Billboard Text That Always Faces the Camera

Spawn `Text2d` in 3D space and rotate it each frame to face the camera.

```rust
use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, billboard_face_camera)
        .run();
}

#[derive(Component)]
struct Billboard;

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // A cube to anchor the label to
    let cube = commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
        MeshMaterial3d(materials.add(Color::srgb(0.2, 0.5, 0.8))),
        Transform::from_xyz(0.0, 0.5, 0.0),
    )).id();

    // Billboard text as a child, floating above the cube
    let font = asset_server.load("fonts/FiraSans-Bold.ttf");
    commands.spawn((
        Text2d::new("Hello!"),
        TextFont {
            font,
            font_size: 36.0,
            ..default()
        },
        TextColor(Color::WHITE),
        Transform::from_xyz(0.0, 1.2, 0.0).with_scale(Vec3::splat(0.01)), // scale down for 3D space
        Billboard,
    )).set_parent(cube);

    // Lighting and camera
    commands.spawn((
        DirectionalLight {
            shadows_enabled: true,
            ..default()
        },
        Transform::default().looking_at(Vec3::new(-1.0, -1.0, -1.0), Vec3::Y),
    ));
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(3.0, 3.0, 3.0).looking_at(Vec3::new(0.0, 0.5, 0.0), Vec3::Y),
    ));
}

fn billboard_face_camera(
    camera_query: Query<&GlobalTransform, With<Camera3d>>,
    mut billboards: Query<&mut Transform, (With<Billboard>, Without<Camera3d>)>,
) {
    let Ok(camera_global) = camera_query.single() else { return };
    let camera_position = camera_global.translation();

    for mut transform in &mut billboards {
        // Compute the direction from the billboard to the camera, ignoring Y to stay upright
        let direction = camera_position - transform.translation;
        if direction.length_squared() > 0.001 {
            transform.look_to(direction, Vec3::Y);
        }
    }
}
```

---

## 6. Render-to-Texture (Camera Rendering to Image Used as Material)

Render a scene from a secondary camera into an image, then apply that image as a texture on a 3D object.

```rust
use bevy::{
    prelude::*,
    render::{
        camera::RenderTarget,
        render_resource::{
            Extent3d, TextureDimension, TextureFormat, TextureUsages,
        },
        view::RenderLayers,
    },
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, rotate_cube)
        .run();
}

#[derive(Component)]
struct RotatingCube;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    // Create the render target image
    let size = Extent3d {
        width: 512,
        height: 512,
        depth_or_array_layers: 1,
    };
    let mut render_image = Image::new_fill(
        size,
        TextureDimension::D2,
        &[0, 0, 0, 255],
        TextureFormat::Bgra8UnormSrgb,
        bevy::render::render_asset::RenderAssetUsages::default(),
    );
    render_image.texture_descriptor.usage =
        TextureUsages::TEXTURE_BINDING
        | TextureUsages::COPY_DST
        | TextureUsages::RENDER_ATTACHMENT;
    let render_image_handle = images.add(render_image);

    // --- Sub-scene rendered by the offscreen camera (layer 1) ---

    // A spinning cube only visible to the offscreen camera
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(1.0, 0.3, 0.1),
            ..default()
        })),
        Transform::from_xyz(0.0, 0.0, 0.0),
        RenderLayers::layer(1),
        RotatingCube,
    ));

    // Light for the sub-scene
    commands.spawn((
        PointLight {
            intensity: 2_000_000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(3.0, 4.0, 3.0),
        RenderLayers::layer(1),
    ));

    // Offscreen camera rendering into the image
    commands.spawn((
        Camera3d::default(),
        Camera {
            target: RenderTarget::Image(render_image_handle.clone().into()),
            clear_color: ClearColorConfig::Custom(Color::srgb(0.1, 0.1, 0.15)),
            ..default()
        },
        Transform::from_xyz(0.0, 2.0, 4.0).looking_at(Vec3::ZERO, Vec3::Y),
        RenderLayers::layer(1),
    ));

    // --- Main scene (layer 0) ---

    // A plane that uses the render texture as its material
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(3.0, 2.0, 0.1))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color_texture: Some(render_image_handle),
            unlit: true,
            ..default()
        })),
        Transform::from_xyz(0.0, 1.0, 0.0),
        RenderLayers::layer(0),
    ));

    // Light for main scene
    commands.spawn((
        PointLight {
            intensity: 1_000_000.0,
            ..default()
        },
        Transform::from_xyz(4.0, 5.0, 4.0),
        RenderLayers::layer(0),
    ));

    // Main camera
    commands.spawn((
        Camera3d::default(),
        Camera {
            order: 1, // render after the offscreen camera
            ..default()
        },
        Transform::from_xyz(0.0, 1.5, 5.0).looking_at(Vec3::new(0.0, 1.0, 0.0), Vec3::Y),
        RenderLayers::layer(0),
    ));
}

fn rotate_cube(time: Res<Time>, mut query: Query<&mut Transform, With<RotatingCube>>) {
    for mut transform in &mut query {
        transform.rotate_y(time.delta_secs() * 1.5);
        transform.rotate_x(time.delta_secs() * 0.7);
    }
}
```
