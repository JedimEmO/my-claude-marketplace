# Bevy Scheduling Guide

Complete reference for Bevy's scheduling system — schedules, ordering, sets, run conditions, and states.

## Built-in Schedules

Bevy runs these schedules in a fixed order each frame:

### Main Schedules (Run Every Frame)

| Schedule | When It Runs | Typical Use |
|----------|-------------|-------------|
| `First` | Very start of each frame | Internal engine bookkeeping, time updates |
| `PreUpdate` | Before `Update` | Engine-level preprocessing (input collection, UI focus) |
| `Update` | Main frame update | **Your game logic goes here** |
| `PostUpdate` | After `Update` | Engine-level postprocessing (transform propagation, rendering sync) |
| `Last` | Very end of each frame | Cleanup, diagnostics |

### Fixed-Timestep Schedules (Run at Fixed Intervals)

These run at a fixed rate (default 64 Hz / every ~15.6ms), independent of frame rate. Multiple ticks can run per frame if the frame was slow, or zero ticks if the frame was fast.

| Schedule | When It Runs | Typical Use |
|----------|-------------|-------------|
| `FixedFirst` | Start of each fixed tick | Fixed-timestep bookkeeping |
| `FixedPreUpdate` | Before `FixedUpdate` | Physics preprocessing |
| `FixedUpdate` | Main fixed tick | **Physics, deterministic gameplay** |
| `FixedPostUpdate` | After `FixedUpdate` | Physics postprocessing, collision detection |
| `FixedLast` | End of each fixed tick | Fixed-timestep cleanup |

### One-Time Schedules

| Schedule | When It Runs | Typical Use |
|----------|-------------|-------------|
| `Startup` | Once, before the first `Update` | Spawning initial entities, loading resources |

### State-Transition Schedules

| Schedule | When It Runs | Typical Use |
|----------|-------------|-------------|
| `OnEnter(state)` | Once, when entering a state | Setup for that state (spawn UI, load level) |
| `OnExit(state)` | Once, when leaving a state | Cleanup (despawn UI, save progress) |
| `OnTransition { exited, entered }` | Once, during a state transition | Logic that depends on both the old and new state |

### Frame Order

Within a single frame, the execution order is:

```
Startup (first frame only)
  |
  v
First -> PreUpdate -> [FixedFirst -> FixedPreUpdate -> FixedUpdate -> FixedPostUpdate -> FixedLast]* -> Update -> PostUpdate -> Last
                       ^--- may run 0, 1, or many times per frame
```

## System Ordering

### Default: Parallel and Unordered

Systems in the same schedule run in **parallel** with **no guaranteed order**, as long as their data access does not conflict. This is Bevy's core performance advantage.

If two systems access the same data mutably, Bevy detects the conflict and runs them sequentially (in arbitrary order).

### Explicit Ordering

```rust
App::new()
    .add_systems(Update, (
        // A runs before B
        system_a.before(system_b),

        // B runs after A (equivalent to above)
        system_b.after(system_a),

        // Chain: runs in order A -> B -> C
        (system_a, system_b, system_c).chain(),

        system_b,
        system_c,
    ))
```

`.before()` and `.after()` accept system names or system sets. `.chain()` is syntactic sugar for chaining `.before()`/`.after()` across a tuple of systems.

### Ambiguity Detection

In debug builds, Bevy warns about "system order ambiguity" when two systems access the same data and have no explicit ordering. Fix by adding `.before()`, `.after()`, `.chain()`, or putting them in ordered sets.

## System Sets

System sets group systems for shared ordering and run conditions.

### Defining Sets

```rust
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
enum GameSet {
    Input,
    Movement,
    Combat,
    UI,
}
```

### Configuring Set Order

```rust
App::new()
    .configure_sets(Update, (
        GameSet::Input,
        GameSet::Movement.after(GameSet::Input),
        GameSet::Combat.after(GameSet::Movement),
        GameSet::UI.after(GameSet::Combat),
    ))
    // Or equivalently with chain:
    .configure_sets(Update, (
        GameSet::Input,
        GameSet::Movement,
        GameSet::Combat,
        GameSet::UI,
    ).chain())
```

### Assigning Systems to Sets

```rust
App::new()
    .add_systems(Update, (
        read_keyboard.in_set(GameSet::Input),
        read_gamepad.in_set(GameSet::Input),
        move_player.in_set(GameSet::Movement),
        move_enemies.in_set(GameSet::Movement),
        deal_damage.in_set(GameSet::Combat),
        apply_damage.in_set(GameSet::Combat),
        update_hud.in_set(GameSet::UI),
    ))
```

### Run Conditions on Sets

Apply a run condition to an entire set — all systems in the set are skipped if the condition is false:

```rust
App::new()
    .configure_sets(Update,
        GameSet::Combat.run_if(in_state(AppState::InGame)),
    )
```

## Run Conditions

Run conditions are functions that return `bool`. If they return `false`, the system (or set) is skipped for that tick.

### Built-in Run Conditions

```rust
use bevy::prelude::*;

// State-based
system.run_if(in_state(AppState::InGame))

// Resource-based
system.run_if(resource_exists::<Score>)
system.run_if(resource_equals(Paused(true)))
system.run_if(resource_changed::<Score>)
system.run_if(resource_added::<Score>)

// Event-based
system.run_if(on_event::<DamageEvent>)

// Time-based (from bevy::time)
system.run_if(on_timer(Duration::from_secs(2)))
system.run_if(on_real_timer(Duration::from_millis(500)))

// Logic combinators
system.run_if(in_state(AppState::InGame).and(resource_exists::<Player>))
system.run_if(in_state(AppState::Paused).or(in_state(AppState::MainMenu)))
system.run_if(not(in_state(AppState::Loading)))
```

### Custom Run Conditions

A run condition is any system that returns `bool`:

```rust
fn has_living_enemies(query: Query<(), With<Enemy>>) -> bool {
    !query.is_empty()
}

fn player_is_alive(query: Query<&Health, With<Player>>) -> bool {
    query.iter().any(|h| h.0 > 0)
}

App::new()
    .add_systems(Update, (
        enemy_ai.run_if(has_living_enemies),
        game_over_check.run_if(not(player_is_alive)),
    ))
```

### Combining Conditions

```rust
App::new()
    .add_systems(Update,
        combat_system
            .run_if(in_state(AppState::InGame))
            .run_if(has_living_enemies)
            // Multiple .run_if() = AND logic (all must be true)
    )
```

## States

States control large-scale game flow: menus, loading, gameplay, pausing.

### Defining States

```rust
#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
enum AppState {
    #[default]
    MainMenu,
    Loading,
    InGame,
    Paused,
    GameOver,
}
```

### Registering and Using States

```rust
App::new()
    .init_state::<AppState>()  // Starts at Default value (MainMenu)
    // OR:
    .insert_state(AppState::Loading)  // Start at a specific value

    // Systems that run once on state entry/exit
    .add_systems(OnEnter(AppState::InGame), setup_game_world)
    .add_systems(OnExit(AppState::InGame), despawn_game_world)

    // Systems that run every frame while in a state
    .add_systems(Update, (
        menu_ui.run_if(in_state(AppState::MainMenu)),
        gameplay.run_if(in_state(AppState::InGame)),
        pause_overlay.run_if(in_state(AppState::Paused)),
    ))
```

### Transitioning Between States

```rust
fn handle_start_button(
    mut next_state: ResMut<NextState<AppState>>,
    interaction: Query<&Interaction, With<StartButton>>,
) {
    for interaction in &interaction {
        if *interaction == Interaction::Pressed {
            next_state.set(AppState::InGame);
        }
    }
}

fn handle_pause(
    mut next_state: ResMut<NextState<AppState>>,
    input: Res<ButtonInput<KeyCode>>,
    state: Res<State<AppState>>,
) {
    if input.just_pressed(KeyCode::Escape) {
        match state.get() {
            AppState::InGame => next_state.set(AppState::Paused),
            AppState::Paused => next_state.set(AppState::InGame),
            _ => {}
        }
    }
}
```

State transitions are applied during `StateTransition` (which runs between `PreUpdate` and `Update`). `OnExit` runs first, then `OnTransition`, then `OnEnter`.

### Sub-States (Bevy 0.15+)

Sub-states only exist when their parent state has a specific value. When the parent leaves that value, the sub-state is removed entirely.

```rust
#[derive(SubStates, Debug, Clone, PartialEq, Eq, Hash, Default)]
#[source(AppState = AppState::InGame)]
enum GamePhase {
    #[default]
    Exploration,
    Combat,
    Cutscene,
}

App::new()
    .init_state::<AppState>()
    .add_sub_state::<GamePhase>()
    .add_systems(OnEnter(GamePhase::Combat), setup_combat_ui)
    .add_systems(OnExit(GamePhase::Combat), cleanup_combat_ui)
    .add_systems(Update, combat_tick.run_if(in_state(GamePhase::Combat)))
```

When `AppState` leaves `InGame`, `GamePhase` is automatically removed. When `AppState` re-enters `InGame`, `GamePhase` is re-initialized to its `Default` value.

### Computed States (Bevy 0.15+)

Computed states derive their value from one or more other states. You cannot set them manually — they update automatically.

```rust
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
enum InCombat {
    Yes,
    No,
}

impl ComputedStates for InCombat {
    type SourceStates = (AppState, Option<GamePhase>);

    fn compute(sources: (AppState, Option<GamePhase>)) -> Option<Self> {
        match sources {
            (AppState::InGame, Some(GamePhase::Combat)) => Some(InCombat::Yes),
            (AppState::InGame, _) => Some(InCombat::No),
            _ => None, // State does not exist outside InGame
        }
    }
}

App::new()
    .init_state::<AppState>()
    .add_sub_state::<GamePhase>()
    .add_computed_state::<InCombat>()
    .add_systems(Update, show_combat_hud.run_if(in_state(InCombat::Yes)))
```
