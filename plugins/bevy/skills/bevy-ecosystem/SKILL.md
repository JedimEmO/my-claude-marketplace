---
name: bevy-ecosystem
description: Use when the user asks about third-party Bevy crates, community plugins, which crate to use for a specific feature, Bevy ecosystem recommendations, or when looking for functionality not built into Bevy core. Also triggers for questions about Bevy version compatibility, migration between versions, or keeping up with breaking changes.
version: 1.0.0
---

# Bevy Ecosystem — Third-Party Crates & Migration

Bevy's core is intentionally lean. The community fills the gaps with high-quality crates for physics, input, networking, UI, and more. This skill helps you choose the right crate, verify version compatibility, and navigate Bevy's fast-moving release cycle.

> For in-depth usage of specific crates, see the dedicated skills:
> - **bevy-physics** — Avian and bevy_rapier setup, colliders, raycasting, joints
> - **bevy-input-and-interaction** — leafwing-input-manager action mapping, input contexts
> - **bevy-ui-and-audio** — bevy_kira_audio advanced audio, bevy_egui debug panels

## Essential Crates Overview

The reference file `references/ecosystem-crates.md` has full dependency lines, setup code, and links. Below is the quick map so you know what exists.

### Physics
| Crate | What it does |
|---|---|
| `avian3d` / `avian2d` | ECS-native physics engine built for Bevy. Preferred for new projects. |
| `bevy_rapier3d` / `bevy_rapier2d` | Rapier physics integration. Mature, widely used. |

### Input
| Crate | What it does |
|---|---|
| `leafwing-input-manager` | Declarative action-to-input mapping with combos, chords, and virtual axes. |

### Assets & Loading
| Crate | What it does |
|---|---|
| `bevy_asset_loader` | Declarative asset loading states — define what to load, get a callback when done. |
| `iyes_progress` | Track loading progress across multiple asset collections. |

### Animation
| Crate | What it does |
|---|---|
| `bevy_tweening` | Tweens and animation sequences for transforms, colors, and custom components. |

### UI & Editor
| Crate | What it does |
|---|---|
| `bevy_egui` | Immediate-mode egui inside Bevy — great for dev tools and debug panels. |
| `bevy_cosmic_edit` | Rich text editing widget powered by cosmic-text. |

### Networking
| Crate | What it does |
|---|---|
| `lightyear` | Client-prediction, server-authoritative networking with rollback. |
| `bevy_replicon` | High-level replication framework — entity spawning, component sync, RPCs. |
| `bevy_renet` | Lower-level reliable UDP transport for Bevy. |

### Debug & Dev Tools
| Crate | What it does |
|---|---|
| `bevy-inspector-egui` | Runtime ECS inspector — browse entities, edit components live. |
| `bevy_screen_diagnostics` | On-screen FPS, entity count, and custom diagnostics overlay. |

### Tilemap & Level Design
| Crate | What it does |
|---|---|
| `bevy_ecs_tilemap` | High-performance ECS-backed tilemap renderer. |

### Particles & VFX
| Crate | What it does |
|---|---|
| `bevy_hanabi` | GPU-accelerated particle system with visual effect graphs. |

### Camera
| Crate | What it does |
|---|---|
| `bevy_pancam` | Plug-and-play 2D camera: pan, zoom, bounds. |
| `bevy_flycam` | Simple 3D fly camera for prototyping and debugging. |

### Persistence & Serialization
| Crate | What it does |
|---|---|
| `bevy_pkv` | Simple key-value store for settings and save data (backed by sled or browser localStorage). |

## Version Compatibility

Bevy releases break things. Every crate must target a specific Bevy version. Here is how to avoid mismatches:

### Check before you `cargo add`

1. **Look at the crate's `Cargo.toml`** or its README — most ecosystem crates have a compatibility table showing which crate version maps to which Bevy version.

2. **Search for the `bevy-tracking` GitHub label.** Many crate repos use labels like `bevy-0.15` or `bevy-tracking` to track PRs that update to the latest Bevy release.

3. **Check the crate's latest release date.** If the crate was last published before the Bevy version you are using shipped, it almost certainly does not support it yet.

4. **Look at `Cargo.toml` dependency specification.** A crate specifying `bevy = "0.15"` works with Bevy 0.15.x but not 0.14 or 0.16.

### What to do when a crate is behind

- Check the crate's `main` branch — an unreleased update may already exist. Use a git dependency temporarily:
  ```toml
  bevy_some_crate = { git = "https://github.com/author/bevy_some_crate", branch = "main" }
  ```
- Search for forks that have already updated.
- Pin your Bevy version to match the crate if the feature is critical.

## Migration Strategy

Bevy does not have a stability guarantee yet. Major releases (0.14 to 0.15, etc.) routinely contain breaking changes. Here is how to handle upgrades:

### Where to find migration guides

The official migration guides live at:
```
https://bevyengine.org/learn/migration-guides/
```

Each guide is organized by the Bevy release (e.g., "0.14 to 0.15") and lists every breaking change with before/after code.

### Common breaking change patterns

- **System parameter changes** — query syntax or resource access patterns change.
- **Plugin API reshuffles** — `add_plugins` signature, plugin group composition.
- **Rendering pipeline changes** — material/shader APIs evolve rapidly.
- **Schedule renaming** — `CoreSet`, `Update`, startup system registration.
- **Asset system changes** — `AssetServer` API, handle types, loading patterns.

### Upgrade strategy

1. **Pin your current Bevy version** in `Cargo.toml` before starting the upgrade so you have a known-good baseline.
2. **Read the full migration guide** for your target version before changing any code.
3. **Bump the Bevy version** in `Cargo.toml` and let the compiler fail.
4. **Fix one system at a time.** The compiler errors are your checklist — each error corresponds to a documented breaking change.
5. **Update third-party crates** to their compatible versions (see Version Compatibility above).
6. **Run your game** after each batch of fixes, not just at the end.

### Tip: compiler-driven migration

Bevy's type system is strict enough that most breaking changes produce compiler errors rather than silent bugs. Trust the compiler. If it compiles and your systems still run, the migration is almost certainly correct.

## Finding New Crates

When you need functionality not listed here:

1. **Bevy Assets page** — the official curated list:
   ```
   https://bevyengine.org/assets/
   ```
   Categorized, searchable, with version compatibility info.

2. **awesome-bevy** — community-maintained GitHub repo:
   ```
   https://github.com/bevyengine/bevy-assets
   ```

3. **crates.io** — search with the `bevy` keyword or category. Most Bevy ecosystem crates use `bevy` as a keyword.

4. **This Week in Bevy** — weekly newsletter covering new crates, updates, and community highlights:
   ```
   https://thisweekinbevy.com/
   ```

When evaluating a crate, check: last commit date, Bevy version support, number of open issues, and whether the maintainer is active in the Bevy Discord.
