# Bevy System Parameters Cheatsheet

Complete reference of all system parameter types available in Bevy 0.15+.

## System Parameters

| Type | Purpose | Example | Notes |
|------|---------|---------|-------|
| `Query<&T>` | Read component data from entities | `Query<&Transform>` | Iterates all entities with `Transform` |
| `Query<&mut T>` | Write component data | `Query<&mut Health>` | Requires `mut query` binding |
| `Query<(&A, &B)>` | Read multiple components | `Query<(&Transform, &Velocity)>` | Only matches entities with both |
| `Query<(&mut A, &B)>` | Mix read and write | `Query<(&mut Transform, &Velocity)>` | Some mutable, some read-only |
| `Query<Entity>` | Get entity IDs only | `Query<Entity, With<Player>>` | Lightweight, no component data fetched |
| `Query<&T, With<U>>` | Read with filter | `Query<&Health, With<Player>>` | Fetch `Health` only from `Player` entities |
| `Query<&T, Without<U>>` | Exclude filter | `Query<&Health, Without<Invincible>>` | Skip entities that have `Invincible` |
| `Query<&T, Changed<T>>` | Changed filter | `Query<&Health, Changed<Health>>` | Only entities whose `Health` changed this tick |
| `Query<&T, Added<T>>` | Added filter | `Query<&Health, Added<Health>>` | Only entities that just received `Health` |
| `Query<&T, (With<A>, Without<B>)>` | Combined filters | `Query<&Health, (With<Enemy>, Without<Shield>)>` | Tuple of filters = AND logic |
| `Query<(&A, Option<&B>)>` | Optional component | `Query<(&Transform, Option<&Velocity>)>` | Matches all with `Transform`; `Velocity` may be `None` |
| `Single<&T>` | Exactly one entity (0.15+) | `Single<&Transform, With<Player>>` | Panics if zero or multiple matches. Use for unique entities |
| `Res<T>` | Read-only resource | `Res<Time>` | Panics if resource does not exist |
| `ResMut<T>` | Mutable resource | `ResMut<Score>` | Requires `mut score` binding |
| `Option<Res<T>>` | Optional resource (read) | `Option<Res<Score>>` | Returns `None` if resource not inserted |
| `Option<ResMut<T>>` | Optional resource (write) | `Option<ResMut<Score>>` | Returns `None` if resource not inserted |
| `Commands` | Deferred world mutations | `Commands` | Spawn, despawn, insert/remove components. Applied between system sets |
| `EventReader<T>` | Read events | `EventReader<DamageEvent>` | Tracks read position automatically. Events persist for 2 frames |
| `EventWriter<T>` | Send events | `EventWriter<DamageEvent>` | Use `.send(event)` to emit |
| `Local<T>` | Per-system local state | `Local<u32>` | Persists across system runs. Each system instance gets its own copy. `T: Default` required |
| `ParamSet<(Q1, Q2)>` | Conflicting queries | `ParamSet<(Query<&mut A, With<B>>, Query<&mut A, Without<B>>)>` | Use when two queries would conflict. Access via `.p0()`, `.p1()` |
| `NonSend<T>` | Non-Send resource (read) | `NonSend<WinitWindows>` | For resources that must stay on the main thread |
| `NonSendMut<T>` | Non-Send resource (write) | `NonSendMut<WinitWindows>` | Forces system to run on main thread |
| `Deferred<T>` | Custom deferred mutations | `Deferred<MyBuffer>` | Batches writes that apply later. `T: SystemBuffer` required |

## Query Filter Types

| Filter | Matches | Example |
|--------|---------|---------|
| `With<T>` | Entities that have component `T` | `Query<&Health, With<Player>>` |
| `Without<T>` | Entities that do NOT have component `T` | `Query<&Health, Without<Invincible>>` |
| `Changed<T>` | Entities whose `T` was mutated this tick | `Query<&Transform, Changed<Transform>>` |
| `Added<T>` | Entities that received `T` this tick | `Query<Entity, Added<Enemy>>` |
| `Or<(F1, F2)>` | Entities matching any filter | `Query<&Name, Or<(With<Player>, With<Ally>)>>` |

## Common Query Patterns

```rust
// Iterate all matches
for (transform, velocity) in &query { }

// Iterate with mutation
for (mut transform, velocity) in &mut query { }

// Get specific entity
if let Ok(health) = query.get(entity) { }
if let Ok(mut health) = query.get_mut(entity) { }

// Check if entity matches
let exists = query.contains(entity);

// Single result (panics if not exactly one)
let player_transform = single_query.into_inner();

// Count matches
let enemy_count = query.iter().count();

// Check if any matches exist
let has_enemies = !query.is_empty();
```

## ParamSet Usage

When two queries in the same system would conflict (both accessing the same component mutably, or one reading and one writing), use `ParamSet`:

```rust
fn system(mut params: ParamSet<(
    Query<&mut Transform, With<Player>>,
    Query<&mut Transform, With<Enemy>>,
)>) {
    // Access one at a time — cannot hold both simultaneously
    for mut transform in params.p0().iter_mut() {
        transform.translation.x += 1.0;
    }
    for mut transform in params.p1().iter_mut() {
        transform.translation.x -= 1.0;
    }
}
```

## Trigger (Observer Systems)

Observer systems use `Trigger<T>` instead of regular system parameters:

```rust
fn on_damage(
    trigger: Trigger<DamageEvent>,
    mut query: Query<&mut Health>,
) {
    let event = trigger.event();
    let target = trigger.target();
    if let Ok(mut health) = query.get_mut(target) {
        health.0 -= event.amount;
    }
}
```
