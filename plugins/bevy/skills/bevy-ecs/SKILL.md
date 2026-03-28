---
name: bevy-ecs
description: Use when the user asks about Bevy's Entity Component System, defining components, writing systems, queries, commands, resources, events, observers, system ordering, system sets, run conditions, or the ECS paradigm in Bevy. Also triggers when the user is confused about the ECS mental model or asks how to structure game logic.
tools: Read, Glob, Grep, Edit, Write, Bash
---

# Bevy ECS — Entity Component System Fundamentals

## Mental Model

Entities are IDs, components are data structs, systems are functions that query components. Composition over inheritance.

- **Entity** — A unique ID (like a database row). No data or behavior by itself.
- **Component** — A plain Rust struct attached to an entity. This is your data.
- **System** — A function that queries entities by component combination. This is your behavior.

```
OOP:  class Player extends Character { hp: i32, speed: f32 }
ECS:  entity.insert((Player, Health(100), Speed(3.0), Transform::default()))
```

Systems run automatically each frame — the scheduler invokes them based on the data they request.

## Components

A component is any Rust type with `#[derive(Component)]`:

```rust
#[derive(Component)]
struct Health(i32);

#[derive(Component)]
struct Speed(f32);

#[derive(Component, Default)]
struct Player;

#[derive(Component)]
struct Enemy {
    aggro_range: f32,
    damage: i32,
}
```

### Marker Components

Zero-sized types used purely for filtering queries:

```rust
#[derive(Component, Default)]
struct Player;

#[derive(Component)]
struct Poisoned;

#[derive(Component)]
struct Grounded;
```

### Required Components (Bevy 0.15+)

Use `#[require(...)]` to auto-insert dependencies when a component is added (uses `Default` unless overridden at spawn):

```rust
#[derive(Component, Default)]
#[require(Health, Speed)]
struct Player;

// commands.spawn(Player) automatically inserts Health::default() and Speed::default()
// commands.spawn((Player, Speed(10.0))) overrides the Speed default
```

### Common Derive Macros

- `Component` — required for all components
- `Debug`, `Clone`, `PartialEq` — commonly combined with `Component`
- `Default` — needed for `#[require]` and `init_resource`
- `Reflect` + `#[reflect(Component)]` — enables runtime inspection (editor tooling, serialization)

## Systems

Systems are plain Rust functions. Their parameters declare what data they need, and Bevy injects the data automatically:

```rust
fn move_entities(mut query: Query<(&mut Transform, &Velocity)>, time: Res<Time>) {
    for (mut transform, velocity) in &mut query {
        transform.translation += velocity.0 * time.delta_secs();
    }
}
```

Register systems when building your app:

```rust
fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, (move_entities, check_health, handle_input))
        .run();
}
```

### System Parameter Types

Key types: `Query`, `Res`/`ResMut`, `Commands`, `EventReader`/`EventWriter`, `Local`, `Single`, `ParamSet`, `Option<Res<T>>`.

> See the **system-params-cheatsheet** reference for the complete table with examples and notes.

## Queries

Queries are how systems access entity data. The type signature determines what data is fetched and how it is filtered.

### Basic Queries

```rust
// Read one component
fn system(query: Query<&Transform>) {
    for transform in &query {
        info!("Position: {}", transform.translation);
    }
}

// Read multiple components
fn system(query: Query<(&Transform, &Health, &Name)>) {
    for (transform, health, name) in &query {
        info!("{} at {} with {} hp", name, transform.translation, health.0);
    }
}

// Write to components
fn system(mut query: Query<(&mut Transform, &Velocity)>) {
    for (mut transform, velocity) in &mut query {
        transform.translation += velocity.0;
    }
}
```

### Query Filters

Filters go in the second type parameter of `Query`:

```rust
// Only entities that have the Player component
fn system(query: Query<&Transform, With<Player>>) { }

// Entities with Health but NOT the Invincible component
fn system(query: Query<&mut Health, Without<Invincible>>) { }

// Entities whose Transform changed since last system run
fn system(query: Query<&Transform, Changed<Transform>>) { }

// Entities that just had the Poisoned component added
fn system(query: Query<Entity, Added<Poisoned>>) { }

// Combine multiple filters with tuples
fn system(query: Query<&mut Health, (With<Enemy>, Without<Shield>)>) { }
```

### Optional Components

Use `Option<&T>` to query entities that may or may not have a component:

```rust
fn system(query: Query<(&Transform, Option<&Velocity>)>) {
    for (transform, maybe_velocity) in &query {
        if let Some(velocity) = maybe_velocity {
            // Entity has velocity
        } else {
            // Entity is stationary
        }
    }
}
```

### Single-Entity Queries

When you expect exactly one matching entity, use `Single<>` (Bevy 0.15+):

```rust
fn camera_follow(
    player: Single<&Transform, With<Player>>,
    mut camera: Single<&mut Transform, With<Camera>>,
) {
    camera.translation = player.translation;
}
```

If zero or more than one entity matches, the system panics. Use this for unique entities like "the player", "the main camera", or "the UI root".

### Querying by Entity ID

```rust
fn system(query: Query<&Health>, specific_entity: Res<TrackedEntity>) {
    if let Ok(health) = query.get(specific_entity.0) {
        info!("Health: {}", health.0);
    }
}
```

## Commands

Commands perform **deferred** world mutations. They do not take effect immediately — they are applied at the end of the current stage (between system sets). This avoids borrow conflicts.

### Spawning Entities

```rust
fn spawn_enemies(mut commands: Commands) {
    // Spawn with a bundle of components
    let entity = commands.spawn((
        Enemy { aggro_range: 10.0, damage: 5 },
        Health(50),
        Transform::default(),
        Visibility::default(),
    )).id();

    // Spawn and then add more components
    commands.spawn((Player, Health(100)))
        .insert(Speed(5.0))
        .insert(Name::new("Hero"));
}
```

### Inserting and Removing Components

```rust
fn poison_system(
    mut commands: Commands,
    query: Query<Entity, (With<Enemy>, Without<Poisoned>)>,
) {
    for entity in &query {
        commands.entity(entity).insert(Poisoned);
    }
}

fn cure_system(
    mut commands: Commands,
    query: Query<Entity, With<Poisoned>>,
) {
    for entity in &query {
        commands.entity(entity).remove::<Poisoned>();
    }
}
```

### Despawning Entities

```rust
fn cleanup_dead(
    mut commands: Commands,
    query: Query<Entity, With<Dead>>,
) {
    for entity in &query {
        // Despawn the entity and all its children
        commands.entity(entity).despawn();
    }
}
```

### Spawning Children (Hierarchies)

```rust
fn spawn_ui(mut commands: Commands) {
    commands.spawn(Node {
        width: Val::Percent(100.0),
        height: Val::Percent(100.0),
        ..default()
    }).with_children(|parent| {
        parent.spawn((
            Text::new("Hello, Bevy!"),
            TextFont {
                font_size: 40.0,
                ..default()
            },
        ));
    });
}
```

## Resources

Resources are global singletons — data that exists once, not per-entity. Use them for game-wide state.

```rust
#[derive(Resource)]
struct Score(u32);

#[derive(Resource, Default)]
struct GameSettings {
    difficulty: Difficulty,
    volume: f32,
}
```

### Inserting Resources

```rust
fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        // Insert with an explicit value
        .insert_resource(Score(0))
        // Insert using Default::default()
        .init_resource::<GameSettings>()
        .run();
}
```

- `insert_resource(value)` — provide a concrete instance.
- `init_resource::<T>()` — requires `T: Default` (or `T: FromWorld`). Creates the resource from its default.

### Accessing Resources in Systems

```rust
fn display_score(score: Res<Score>) {
    info!("Current score: {}", score.0);
}

fn increment_score(mut score: ResMut<Score>) {
    score.0 += 10;
}
```

### Optional Resources

If a resource might not exist:

```rust
fn system(score: Option<Res<Score>>) {
    if let Some(score) = score {
        info!("Score: {}", score.0);
    }
}
```

## Events and Observers

### Events

Events are the primary way to communicate between systems without tight coupling.

```rust
#[derive(Event)]
struct DamageEvent {
    entity: Entity,
    amount: i32,
}

#[derive(Event)]
struct GameOverEvent;
```

Register events and use `EventWriter` / `EventReader`:

```rust
fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_event::<DamageEvent>()
        .add_event::<GameOverEvent>()
        .add_systems(Update, (deal_damage, apply_damage).chain())
        .run();
}

fn deal_damage(
    mut writer: EventWriter<DamageEvent>,
    query: Query<(Entity, &ContactInfo), With<Hazard>>,
) {
    for (entity, contact) in &query {
        writer.send(DamageEvent {
            entity: contact.other_entity,
            amount: 10,
        });
    }
}

fn apply_damage(
    mut reader: EventReader<DamageEvent>,
    mut query: Query<&mut Health>,
) {
    for event in reader.read() {
        if let Ok(mut health) = query.get_mut(event.entity) {
            health.0 -= event.amount;
        }
    }
}
```

Events last for **two frames** by default, then are dropped. Always read events every frame to avoid missing them.

### Observers (Bevy 0.15+)

Observers are reactive — they run immediately when a specific event is triggered, without waiting for the schedule. They are ideal for structural changes.

```rust
#[derive(Event)]
struct OnDeath;

fn setup(mut commands: Commands) {
    commands.spawn((
        Enemy { aggro_range: 10.0, damage: 5 },
        Health(50),
    )).observe(on_death);
}

fn on_death(trigger: Trigger<OnDeath>, mut commands: Commands) {
    // `trigger.target()` is the entity that the event was triggered on
    let entity = trigger.target();
    commands.entity(entity).despawn();
    info!("Entity {:?} died", entity);
}
```

Trigger an observer:

```rust
fn check_health(
    mut commands: Commands,
    query: Query<(Entity, &Health), Changed<Health>>,
) {
    for (entity, health) in &query {
        if health.0 <= 0 {
            commands.trigger_targets(OnDeath, entity);
        }
    }
}
```

Global observers (not tied to a specific entity):

```rust
fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_observer(on_any_death)
        .run();
}

fn on_any_death(trigger: Trigger<OnDeath>, mut score: ResMut<Score>) {
    score.0 += 100;
}
```

### One-Shot Systems

Run a system once on demand using `Commands`:

```rust
fn trigger_explosion(mut commands: Commands) {
    commands.run_system(explosion_effect);
}

fn explosion_effect(mut query: Query<&mut Health, With<Enemy>>) {
    for mut health in &mut query {
        health.0 -= 50;
    }
}
```

## Scheduling

Register systems into schedules: `Startup` (once), `Update` (every frame), `FixedUpdate` (fixed timestep, default 64 Hz). Systems in the same schedule run in parallel by default.

Enforce ordering with `.before()`, `.after()`, or `.chain()`:

```rust
App::new()
    .add_systems(Startup, setup)
    .add_systems(Update, (
        (read_input, move_player, check_collisions).chain(),
        game_logic.run_if(in_state(AppState::InGame)),
    ))
```

Use **system sets** to group systems with shared ordering and run conditions. Use **states** (`States`, `SubStates`) to control which systems run based on app phase, with `OnEnter`/`OnExit` schedules for setup and cleanup.

> See the **scheduling-guide** reference for all built-in schedules, ordering primitives, run conditions, and state patterns.

## Writing Custom Plugins

Plugins are the standard way to organize related systems, resources, and events into reusable modules:

```rust
pub struct CombatPlugin;

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<DamageEvent>()
            .add_event::<DeathEvent>()
            .init_resource::<CombatStats>()
            .add_systems(Update, (
                deal_damage,
                apply_damage,
                check_death,
            ).chain().in_set(GameSet::Combat));
    }
}
```

Use plugins in your app:

```rust
fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins((
            CombatPlugin,
            InventoryPlugin,
            AudioPlugin,
        ))
        .run();
}
```

### Plugin Groups

Group multiple plugins together:

```rust
pub struct GamePlugins;

impl PluginGroup for GamePlugins {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(CombatPlugin)
            .add(InventoryPlugin)
            .add(MovementPlugin)
            .add(UIPlugin)
    }
}

// Use like DefaultPlugins:
fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(GamePlugins)
        .run();
}
```

### Configurable Plugins

Accept configuration by storing it in the plugin struct:

```rust
pub struct PhysicsPlugin {
    pub gravity: f32,
    pub substeps: u32,
}

impl Default for PhysicsPlugin {
    fn default() -> Self {
        Self {
            gravity: -9.81,
            substeps: 4,
        }
    }
}

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(PhysicsConfig {
            gravity: self.gravity,
            substeps: self.substeps,
        });
        app.add_systems(FixedUpdate, (
            apply_gravity,
            resolve_collisions,
        ).chain());
    }
}
```
