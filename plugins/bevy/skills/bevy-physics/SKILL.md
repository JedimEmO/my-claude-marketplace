---
name: bevy-physics
description: Use when the user asks about physics simulation in Bevy, collision detection, rigid bodies, colliders, raycasting for physics, joints, character controllers, or integrating avian or bevy_rapier physics.
version: 1.0.0
---

# Bevy Physics — Rigid Bodies, Colliders & Simulation

> **Related skills**: See `bevy-ecs` for the Entity Component System fundamentals that underpin all physics components and systems. See `bevy-ecosystem` for other third-party crate recommendations beyond physics.

## Crate Choice

There are two physics ecosystems for Bevy. Pick one per project — do not mix them.

| | **avian** (avian2d / avian3d) | **bevy_rapier** (bevy_rapier2d / bevy_rapier3d) |
|---|---|---|
| Engine | Pure Rust, built for Bevy from scratch | Rust wrapper around the rapier engine |
| Recommendation | **Recommended for new projects** | Mature, battle-tested, larger community backlog |
| API style | Bevy-native components and resources | Thin Bevy wrapper over rapier types |
| Determinism | Designed for cross-platform determinism | Deterministic within same platform |

See `references/physics-comparison.md` for a full side-by-side API comparison.

## avian Setup

### Cargo.toml

```toml
# For 3D physics:
[dependencies]
avian3d = "0.2"

# For 2D physics:
[dependencies]
avian2d = "0.2"
```

### Plugin Registration

```rust
use avian3d::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(PhysicsPlugins::default())
        // Optional: visual debug overlay
        .add_plugins(PhysicsDebugPlugin::default())
        .run();
}
```

### Core Components

```rust
use avian3d::prelude::*;

fn spawn_dynamic_body(mut commands: Commands) {
    commands.spawn((
        // Physics role — Dynamic bodies are affected by forces and gravity
        RigidBody::Dynamic,
        // Shape used for collision detection
        Collider::sphere(0.5),
        // Movement
        LinearVelocity(Vec3::new(2.0, 0.0, 0.0)),
        AngularVelocity(Vec3::new(0.0, 1.0, 0.0)),
        // Physical properties
        Mass(1.0),
        Restitution::new(0.7),  // Bounciness: 0.0 = no bounce, 1.0 = perfect bounce
        Friction::new(0.5),
        GravityScale(1.0),      // 0.0 = no gravity, 2.0 = double gravity
        // Spatial placement
        Transform::from_xyz(0.0, 5.0, 0.0),
    ));
}

fn spawn_static_floor(mut commands: Commands) {
    commands.spawn((
        RigidBody::Static,  // Never moves, infinite mass
        Collider::cuboid(50.0, 0.5, 50.0),
        Transform::default(),
    ));
}

fn spawn_kinematic_platform(mut commands: Commands) {
    commands.spawn((
        RigidBody::Kinematic,  // Moved by code, not by physics
        Collider::cuboid(5.0, 0.5, 5.0),
        Transform::from_xyz(0.0, 2.0, 0.0),
    ));
}
```

**RigidBody types**:
- `Dynamic` — Fully simulated (gravity, forces, collisions move it)
- `Static` — Immovable (floors, walls). Never set velocity on these.
- `Kinematic` — Moved by code via `Transform` or `LinearVelocity`, but not affected by forces or gravity. Use for moving platforms, elevators.

## bevy_rapier Setup

### Cargo.toml

```toml
# For 3D physics:
[dependencies]
bevy_rapier3d = "0.28"

# For 2D physics:
[dependencies]
bevy_rapier2d = "0.28"
```

### Plugin Registration

```rust
use bevy_rapier3d::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        // Optional: wireframe debug rendering for colliders
        .add_plugins(RapierDebugRenderPlugin::default())
        .run();
}
```

### Core Components

```rust
use bevy_rapier3d::prelude::*;

fn spawn_dynamic_body(mut commands: Commands) {
    commands.spawn((
        RigidBody::Dynamic,
        Collider::ball(0.5),
        Velocity {
            linvel: Vec3::new(2.0, 0.0, 0.0),
            angvel: Vec3::new(0.0, 1.0, 0.0),
        },
        ExternalForce {
            force: Vec3::ZERO,
            torque: Vec3::ZERO,
        },
        Damping {
            linear_damping: 0.5,
            angular_damping: 0.1,
        },
        Restitution::coefficient(0.7),
        Friction::coefficient(0.5),
        ColliderMassProperties::Mass(1.0),
        Transform::from_xyz(0.0, 5.0, 0.0),
    ));
}
```

## Collider Shapes

Both crates support the same fundamental shapes with slightly different syntax.

**Important**: Cuboid dimensions are **half-extents**, not full dimensions. `Collider::cuboid(1.0, 1.0, 1.0)` creates a 2x2x2 box.

**Capsule parameter differences**: avian uses `capsule(radius, length)` while rapier uses `capsule_y(half_height, radius)` — note both the naming and parameter order differ.

```rust
// ---- avian3d ----
Collider::sphere(0.5)                     // radius
Collider::cuboid(1.0, 2.0, 1.0)          // half-extents x, y, z
Collider::capsule(0.5, 1.0)              // radius, length
Collider::cylinder(0.5, 2.0)             // radius, height
Collider::cone(0.5, 1.0)                 // radius, height
Collider::triangle(a, b, c)              // three Vec3 vertices
Collider::trimesh_from_mesh(&mesh)        // arbitrary triangle mesh
Collider::convex_hull(points)             // convex hull from point cloud
Collider::compound(vec![                  // multiple shapes combined
    (Vec3::ZERO, Quat::IDENTITY, Collider::sphere(0.5)),
    (Vec3::new(0.0, 1.0, 0.0), Quat::IDENTITY, Collider::cuboid(0.3, 0.3, 0.3)),
])

// ---- bevy_rapier3d ----
Collider::ball(0.5)                       // radius
Collider::cuboid(1.0, 2.0, 1.0)          // half-extents x, y, z
Collider::capsule_y(1.0, 0.5)            // half-height, radius
Collider::cylinder(1.0, 0.5)             // half-height, radius
Collider::cone(1.0, 0.5)                 // half-height, radius
Collider::triangle(a, b, c)              // three Vec3 vertices
Collider::trimesh(vertices, indices)      // vertices + triangle indices
Collider::convex_hull(&points)            // convex hull from point cloud
Collider::compound(vec![                  // multiple shapes combined
    (Vec3::ZERO, Quat::IDENTITY, Collider::ball(0.5)),
    (Vec3::new(0.0, 1.0, 0.0), Quat::IDENTITY, Collider::cuboid(0.3, 0.3, 0.3)),
])
```

**2D equivalents**: In avian2d, use `Collider::circle(r)`, `Collider::rectangle(w, h)`, `Collider::capsule(r, l)`. In bevy_rapier2d, use `Collider::ball(r)`, `Collider::cuboid(hx, hy)`, `Collider::capsule_y(hl, r)`.

## Collision Detection

### Collision Events

```rust
// ---- avian ----
use avian3d::prelude::*;

fn handle_collisions(
    mut collision_started: EventReader<CollisionStarted>,
    mut collision_ended: EventReader<CollisionEnded>,
) {
    for CollisionStarted(entity_a, entity_b) in collision_started.read() {
        println!("{entity_a:?} started colliding with {entity_b:?}");
    }
    for CollisionEnded(entity_a, entity_b) in collision_ended.read() {
        println!("{entity_a:?} stopped colliding with {entity_b:?}");
    }
}

// ---- bevy_rapier ----
use bevy_rapier3d::prelude::*;

fn handle_collisions(mut collision_events: EventReader<CollisionEvent>) {
    for event in collision_events.read() {
        match event {
            CollisionEvent::Started(a, b, _flags) => {
                println!("{a:?} started colliding with {b:?}");
            }
            CollisionEvent::Stopped(a, b, _flags) => {
                println!("{a:?} stopped colliding with {b:?}");
            }
        }
    }
}
```

### Collision Layers and Groups

Collision layers let you control which objects can collide with which:

```rust
// ---- avian ----
use avian3d::prelude::*;

#[derive(PhysicsLayer, Default)]
enum GameLayer {
    #[default]
    Default,
    Player,
    Enemy,
    Projectile,
    Terrain,
}

// Player collides with enemies and terrain, but not other players
commands.spawn((
    RigidBody::Dynamic,
    Collider::capsule(0.4, 1.0),
    CollisionLayers::new(GameLayer::Player, [GameLayer::Enemy, GameLayer::Terrain]),
));

// ---- bevy_rapier ----
use bevy_rapier3d::prelude::*;

const PLAYER_GROUP: Group = Group::GROUP_1;
const ENEMY_GROUP: Group = Group::GROUP_2;
const TERRAIN_GROUP: Group = Group::GROUP_3;

commands.spawn((
    RigidBody::Dynamic,
    Collider::capsule_y(0.5, 0.4),
    CollisionGroups::new(PLAYER_GROUP, ENEMY_GROUP | TERRAIN_GROUP),
));
```

### Sensors (Trigger Volumes)

Sensors detect overlap without generating physical contact responses. Use them for trigger zones, pickup areas, and damage regions.

```rust
// ---- avian ----
commands.spawn((
    Collider::sphere(3.0),
    Sensor,  // No physical response, just generates events
    CollisionLayers::new(GameLayer::Default, GameLayer::Player),
));

// ---- bevy_rapier ----
commands.spawn((
    Collider::ball(3.0),
    Sensor,
    ActiveEvents::COLLISION_EVENTS,  // Required to receive events for sensors
));
```

## Raycasting

### avian — RayCaster Component and SpatialQuery

```rust
use avian3d::prelude::*;

// Approach 1: RayCaster component (persists, updates each frame)
commands.spawn((
    RayCaster::new(Vec3::ZERO, Direction3d::NEG_Y)
        .with_max_distance(100.0),
    Transform::from_xyz(0.0, 10.0, 0.0),
));

fn read_raycast_hits(query: Query<(&RayCaster, &RayHits)>) {
    for (ray, hits) in &query {
        for hit in hits.iter() {
            println!("Hit entity {:?} at distance {}", hit.entity, hit.distance);
        }
    }
}

// Approach 2: SpatialQuery (one-shot, on-demand)
fn cast_ray_on_demand(spatial_query: SpatialQuery) {
    if let Some(hit) = spatial_query.cast_ray(
        Vec3::new(0.0, 10.0, 0.0),  // origin
        Direction3d::NEG_Y,           // direction
        100.0,                        // max distance
        true,                         // solid (hit interior of shapes)
        &SpatialQueryFilter::default(),
    ) {
        println!("Hit {:?} at distance {}", hit.entity, hit.distance);
    }
}
```

### bevy_rapier — RapierContext

```rust
use bevy_rapier3d::prelude::*;

fn cast_ray(rapier_context: Res<RapierContext>) {
    if let Some((entity, distance)) = rapier_context.cast_ray(
        Vec3::new(0.0, 10.0, 0.0),  // origin
        Vec3::NEG_Y,                  // direction
        100.0,                        // max distance
        true,                         // solid
        QueryFilter::default(),
    ) {
        println!("Hit {entity:?} at distance {distance}");
    }
}

// With hit normal:
fn cast_ray_with_normal(rapier_context: Res<RapierContext>) {
    if let Some((entity, intersection)) = rapier_context.cast_ray_and_get_normal(
        Vec3::new(0.0, 10.0, 0.0),
        Vec3::NEG_Y,
        100.0,
        true,
        QueryFilter::default(),
    ) {
        println!("Hit {entity:?}, normal: {}", intersection.normal);
    }
}
```

## Character Controllers

A character controller is a kinematic body that slides along surfaces, steps over small obstacles, and detects ground contact. Both crates provide built-in character controller components.

### avian

```rust
use avian3d::prelude::*;

fn spawn_character(mut commands: Commands) {
    commands.spawn((
        RigidBody::Kinematic,
        Collider::capsule(0.4, 1.0),
        CharacterController,
        // Movement is applied via LinearVelocity on kinematic bodies
        LinearVelocity::default(),
        Transform::from_xyz(0.0, 1.0, 0.0),
    ));
}

fn move_character(
    mut query: Query<(&mut LinearVelocity, &CharacterController), With<CharacterController>>,
    input: Res<ButtonInput<KeyCode>>,
) {
    for (mut velocity, _controller) in &mut query {
        let mut direction = Vec3::ZERO;
        if input.pressed(KeyCode::KeyW) { direction.z -= 1.0; }
        if input.pressed(KeyCode::KeyS) { direction.z += 1.0; }
        if input.pressed(KeyCode::KeyA) { direction.x -= 1.0; }
        if input.pressed(KeyCode::KeyD) { direction.x += 1.0; }

        let speed = 5.0;
        velocity.0 = direction.normalize_or_zero() * speed;
    }
}
```

### bevy_rapier

```rust
use bevy_rapier3d::prelude::*;

fn spawn_character(mut commands: Commands) {
    commands.spawn((
        RigidBody::KinematicPositionBased,
        Collider::capsule_y(0.5, 0.4),
        KinematicCharacterController {
            offset: CharacterLength::Absolute(0.01),
            max_slope_climb_angle: std::f32::consts::FRAC_PI_4, // 45 degrees
            min_slope_slide_angle: std::f32::consts::FRAC_PI_4,
            snap_to_ground: Some(CharacterLength::Absolute(0.2)),
            ..default()
        },
        Transform::from_xyz(0.0, 1.0, 0.0),
    ));
}

fn move_character(
    mut query: Query<&mut KinematicCharacterController>,
    input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
) {
    for mut controller in &mut query {
        let mut direction = Vec3::ZERO;
        if input.pressed(KeyCode::KeyW) { direction.z -= 1.0; }
        if input.pressed(KeyCode::KeyS) { direction.z += 1.0; }
        if input.pressed(KeyCode::KeyA) { direction.x -= 1.0; }
        if input.pressed(KeyCode::KeyD) { direction.x += 1.0; }

        let speed = 5.0;
        controller.translation = Some(direction.normalize_or_zero() * speed * time.delta_secs());
    }
}

// Ground detection via KinematicCharacterControllerOutput
fn check_grounded(query: Query<&KinematicCharacterControllerOutput>) {
    for output in &query {
        if output.grounded {
            println!("Character is on the ground");
        }
    }
}
```

## Joints

Joints constrain how two rigid bodies move relative to each other.

### avian

```rust
use avian3d::prelude::*;

// Fixed joint — bodies stay rigidly attached
let entity_a = commands.spawn((RigidBody::Dynamic, Collider::sphere(0.5))).id();
let entity_b = commands.spawn((RigidBody::Dynamic, Collider::sphere(0.5))).id();
commands.spawn(FixedJoint::new(entity_a, entity_b));

// Revolute joint — rotation around a single axis (hinge)
commands.spawn(
    RevoluteJoint::new(entity_a, entity_b)
        .with_aligned_axis(Vec3::Z)   // axis of rotation
        .with_angle_limits(-1.0, 1.0) // radians
);

// Prismatic joint — sliding along a single axis (piston/slider)
commands.spawn(
    PrismaticJoint::new(entity_a, entity_b)
        .with_free_axis(Vec3::Y)
        .with_limits(0.0, 5.0) // min/max translation
);

// Distance/spring joint — keeps bodies within a distance range
commands.spawn(
    DistanceJoint::new(entity_a, entity_b)
        .with_limits(1.0, 5.0)
        .with_compliance(0.001) // lower = stiffer spring
);
```

### bevy_rapier

```rust
use bevy_rapier3d::prelude::*;

let entity_a = commands.spawn((RigidBody::Dynamic, Collider::ball(0.5))).id();
let entity_b = commands.spawn((RigidBody::Dynamic, Collider::ball(0.5))).id();

// Fixed joint
commands.spawn(ImpulseJoint::new(
    entity_a,
    FixedJointBuilder::new().local_anchor1(Vec3::ZERO).local_anchor2(Vec3::new(0.0, -1.0, 0.0)),
)).insert(ImpulseJoint::new(entity_a, FixedJointBuilder::new()));

// Revolute joint
let revolute = RevoluteJointBuilder::new(Vec3::Z)
    .local_anchor1(Vec3::new(1.0, 0.0, 0.0))
    .local_anchor2(Vec3::new(-1.0, 0.0, 0.0))
    .limits([-1.0, 1.0]);
commands.entity(entity_b).insert(ImpulseJoint::new(entity_a, revolute));

// Prismatic joint
let prismatic = PrismaticJointBuilder::new(Vec3::Y)
    .local_anchor1(Vec3::ZERO)
    .local_anchor2(Vec3::ZERO)
    .limits([0.0, 5.0]);
commands.entity(entity_b).insert(ImpulseJoint::new(entity_a, prismatic));

// Spring joint (via rapier's SpringJointBuilder if available, or via motor on prismatic)
let spring = SpringJointBuilder::new(2.0, 0.5, 0.1); // rest_length, stiffness, damping
commands.entity(entity_b).insert(ImpulseJoint::new(entity_a, spring));
```

## 2D vs 3D

Each physics crate ships as two separate crates — one for 2D and one for 3D. You cannot mix them in the same Bevy app.

| Dimension | avian | bevy_rapier |
|---|---|---|
| 2D | `avian2d` | `bevy_rapier2d` |
| 3D | `avian3d` | `bevy_rapier3d` |

Key differences in 2D mode:
- Positions use `Vec2`, rotations use scalar angles (radians) instead of `Quat`
- Collider shapes: `circle` instead of `sphere`, `rectangle` instead of `cuboid`
- Gravity default is `(0.0, -9.81)` as a `Vec2`
- Joints rotate around the implicit Z axis
- `LinearVelocity` and `AngularVelocity` use 2D types

Choose 2D physics when your game is truly 2D (platformer, top-down). If you have a 2D game with a 3D camera or 3D models rendered from a fixed angle, you may still want 2D physics for simplicity.
