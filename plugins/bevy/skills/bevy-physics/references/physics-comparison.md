# avian vs bevy_rapier — Side-by-Side Comparison

## Cargo.toml Dependencies

| Dimension | avian | bevy_rapier |
|---|---|---|
| 2D | `avian2d = "0.2"` | `bevy_rapier2d = "0.28"` |
| 3D | `avian3d = "0.2"` | `bevy_rapier3d = "0.28"` |

## Plugin Setup

```rust
// ---- avian3d ----
use avian3d::prelude::*;
App::new()
    .add_plugins(DefaultPlugins)
    .add_plugins(PhysicsPlugins::default())
    .add_plugins(PhysicsDebugPlugin::default())  // optional debug

// ---- bevy_rapier3d ----
use bevy_rapier3d::prelude::*;
App::new()
    .add_plugins(DefaultPlugins)
    .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
    .add_plugins(RapierDebugRenderPlugin::default())  // optional debug
```

## RigidBody Types

| Role | avian | bevy_rapier |
|---|---|---|
| Fully simulated | `RigidBody::Dynamic` | `RigidBody::Dynamic` |
| Immovable | `RigidBody::Static` | `RigidBody::Fixed` |
| Code-driven (transform) | `RigidBody::Kinematic` | `RigidBody::KinematicPositionBased` |
| Code-driven (velocity) | `RigidBody::Kinematic` | `RigidBody::KinematicVelocityBased` |

Note: avian uses a single `Kinematic` variant; bevy_rapier splits it into position-based and velocity-based.

## Collider Creation

| Shape | avian3d | bevy_rapier3d |
|---|---|---|
| Sphere | `Collider::sphere(radius)` | `Collider::ball(radius)` |
| Box | `Collider::cuboid(hx, hy, hz)` | `Collider::cuboid(hx, hy, hz)` |
| Capsule | `Collider::capsule(radius, length)` | `Collider::capsule_y(half_height, radius)` |
| Cylinder | `Collider::cylinder(radius, height)` | `Collider::cylinder(half_height, radius)` |
| Cone | `Collider::cone(radius, height)` | `Collider::cone(half_height, radius)` |
| Triangle mesh | `Collider::trimesh_from_mesh(&mesh)` | `Collider::trimesh(vertices, indices)` |
| Convex hull | `Collider::convex_hull(points)` | `Collider::convex_hull(&points)` |

Note the parameter order difference: avian generally takes `(radius, length)` while rapier takes `(half_height, radius)`.

## Velocity and Force API

| Concept | avian | bevy_rapier |
|---|---|---|
| Linear velocity | `LinearVelocity(Vec3)` component | `Velocity { linvel, angvel }` component |
| Angular velocity | `AngularVelocity(Vec3)` component | `Velocity { linvel, angvel }` component |
| External force | `ExternalForce::new(Vec3)` | `ExternalForce { force, torque }` |
| External impulse | `ExternalImpulse::new(Vec3)` | `ExternalImpulse { impulse, torque_impulse }` |
| Damping | `LinearDamping(f32)`, `AngularDamping(f32)` | `Damping { linear_damping, angular_damping }` |
| Mass | `Mass(f32)` | `ColliderMassProperties::Mass(f32)` |
| Gravity scale | `GravityScale(f32)` | `GravityScale(f32)` |
| Restitution | `Restitution::new(0.7)` | `Restitution::coefficient(0.7)` |
| Friction | `Friction::new(0.5)` | `Friction::coefficient(0.5)` |

Key difference: avian uses separate components for linear and angular velocity; rapier bundles them into a single `Velocity` struct.

## Collision Events

```rust
// ---- avian ----
fn collisions(
    mut started: EventReader<CollisionStarted>,
    mut ended: EventReader<CollisionEnded>,
) {
    for CollisionStarted(a, b) in started.read() { /* ... */ }
    for CollisionEnded(a, b) in ended.read() { /* ... */ }
}

// ---- bevy_rapier ----
fn collisions(mut events: EventReader<CollisionEvent>) {
    for event in events.read() {
        match event {
            CollisionEvent::Started(a, b, _flags) => { /* ... */ }
            CollisionEvent::Stopped(a, b, _flags) => { /* ... */ }
        }
    }
}
// Note: rapier requires `ActiveEvents::COLLISION_EVENTS` on at least one entity.
```

## Collision Layers

```rust
// ---- avian ----
#[derive(PhysicsLayer, Default)]
enum GameLayer { #[default] Default, Player, Enemy }
CollisionLayers::new(GameLayer::Player, [GameLayer::Enemy, GameLayer::Default])

// ---- bevy_rapier ----
const PLAYER: Group = Group::GROUP_1;
const ENEMY: Group = Group::GROUP_2;
CollisionGroups::new(PLAYER, ENEMY | Group::ALL)
```

avian uses a derive macro for named layers. rapier uses bitflag groups.

## Sensors

```rust
// ---- avian ----
commands.spawn((Collider::sphere(3.0), Sensor));

// ---- bevy_rapier ----
commands.spawn((Collider::ball(3.0), Sensor, ActiveEvents::COLLISION_EVENTS));
// rapier requires ActiveEvents for sensors to generate events.
```

## Raycasting

```rust
// ---- avian (one-shot) ----
fn raycast(spatial_query: SpatialQuery) {
    if let Some(hit) = spatial_query.cast_ray(
        origin, direction, max_distance, solid, &SpatialQueryFilter::default()
    ) {
        // hit.entity, hit.distance
    }
}

// ---- avian (persistent component) ----
commands.spawn(RayCaster::new(Vec3::ZERO, Direction3d::NEG_Y).with_max_distance(100.0));
// Read results from RayHits component each frame.

// ---- bevy_rapier ----
fn raycast(rapier_context: Res<RapierContext>) {
    if let Some((entity, distance)) = rapier_context.cast_ray(
        origin, direction, max_distance, solid, QueryFilter::default()
    ) {
        // entity, distance
    }
}
```

avian offers both a persistent `RayCaster` component and an on-demand `SpatialQuery` system parameter. rapier provides only the on-demand `RapierContext` approach.

## Character Controller

```rust
// ---- avian ----
commands.spawn((
    RigidBody::Kinematic,
    Collider::capsule(0.4, 1.0),
    CharacterController,
    LinearVelocity::default(),
));
// Move by setting LinearVelocity directly.

// ---- bevy_rapier ----
commands.spawn((
    RigidBody::KinematicPositionBased,
    Collider::capsule_y(0.5, 0.4),
    KinematicCharacterController {
        max_slope_climb_angle: std::f32::consts::FRAC_PI_4,
        snap_to_ground: Some(CharacterLength::Absolute(0.2)),
        ..default()
    },
));
// Move by setting controller.translation = Some(movement_vector).
// Read KinematicCharacterControllerOutput for grounded state.
```

## Joints

| Joint type | avian | bevy_rapier |
|---|---|---|
| Fixed | `FixedJoint::new(a, b)` | `ImpulseJoint::new(a, FixedJointBuilder::new())` |
| Revolute (hinge) | `RevoluteJoint::new(a, b).with_aligned_axis(axis)` | `ImpulseJoint::new(a, RevoluteJointBuilder::new(axis))` |
| Prismatic (slider) | `PrismaticJoint::new(a, b).with_free_axis(axis)` | `ImpulseJoint::new(a, PrismaticJointBuilder::new(axis))` |
| Spring/distance | `DistanceJoint::new(a, b).with_limits(min, max)` | `ImpulseJoint::new(a, SpringJointBuilder::new(...))` |

avian spawns joints as their own entities. rapier inserts `ImpulseJoint` as a component on one of the two bodies.

## Key Differences and Tradeoffs

| Aspect | avian | bevy_rapier |
|---|---|---|
| **Architecture** | Pure Rust, designed for Bevy from day one | Rust wrapper around the rapier C-like engine |
| **API ergonomics** | More Bevy-idiomatic (separate components, derive macros) | Thin wrapper — API mirrors rapier's own types |
| **Maturity** | Newer, rapidly evolving | Older, more community resources and examples |
| **Performance** | Competitive; benefits from Bevy's parallelism natively | Mature optimizations; well-tuned broadphase |
| **Determinism** | Cross-platform deterministic by design | Deterministic within the same platform/build |
| **Debug rendering** | `PhysicsDebugPlugin` | `RapierDebugRenderPlugin` |
| **Community** | Growing; fewer tutorials/examples available | Larger ecosystem of tutorials, examples, and users |

## When to Choose Which

**Choose avian when:**
- Starting a new Bevy project with no existing rapier code
- You value Bevy-native, idiomatic component APIs
- Cross-platform determinism matters (e.g., lockstep multiplayer)
- You want a single Rust dependency with no C/C++ in the chain

**Choose bevy_rapier when:**
- Migrating from an existing rapier-based project
- You need a specific rapier feature not yet in avian
- You want the largest possible pool of community examples and StackOverflow answers
- Your team already knows the rapier API from other engines
