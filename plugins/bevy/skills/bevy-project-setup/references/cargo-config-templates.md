# Cargo Config Templates for Bevy

Copy-pasteable `.cargo/config.toml` templates for fast Bevy compile times.

## Linux (mold linker)

Install mold: `sudo apt install mold` or `sudo pacman -S mold`

```toml
# .cargo/config.toml

[target.x86_64-unknown-linux-gnu]
linker = "clang"
rustflags = ["-C", "link-arg=-fuse-ld=mold"]
```

## macOS

macOS uses the default linker. No special config is strictly needed, but these flags help:

```toml
# .cargo/config.toml

[target.aarch64-apple-darwin]
rustflags = [
    "-C", "link-arg=-fuse-ld=/usr/bin/ld",
    "-Zshare-generics=y",
]

[target.x86_64-apple-darwin]
rustflags = [
    "-C", "link-arg=-fuse-ld=/usr/bin/ld",
    "-Zshare-generics=y",
]
```

> Note: `-Zshare-generics=y` requires nightly. Remove it if using stable.

## Windows (rust-lld)

```toml
# .cargo/config.toml

[target.x86_64-pc-windows-msvc]
linker = "rust-lld.exe"
```

## Cross-platform (auto-detect)

A single config that works on all platforms by using environment-specific overrides:

```toml
# .cargo/config.toml

# Linux
[target.x86_64-unknown-linux-gnu]
linker = "clang"
rustflags = ["-C", "link-arg=-fuse-ld=mold"]

# macOS ARM
[target.aarch64-apple-darwin]
rustflags = ["-C", "link-arg=-fuse-ld=/usr/bin/ld"]

# macOS x86
[target.x86_64-apple-darwin]
rustflags = ["-C", "link-arg=-fuse-ld=/usr/bin/ld"]

# Windows
[target.x86_64-pc-windows-msvc]
linker = "rust-lld.exe"
```

## Cargo.toml Profile Settings

Add these to your workspace root `Cargo.toml`:

```toml
# Optimize all dependencies in dev mode so the game is playable
# while keeping your own code at opt-level 0 for fast compilation.
[profile.dev.package."*"]
opt-level = 2

# Enable a small amount of optimization in dev mode for your code.
# Remove this if compile times are more important than dev frame rates.
[profile.dev]
opt-level = 1

# Release profile: maximum performance
[profile.release]
lto = "thin"
codegen-units = 1
opt-level = 3

# Stripped release build for distribution (cargo build --profile dist)
[profile.dist]
inherits = "release"
lto = "fat"
strip = true
```

## Cranelift Backend (Nightly Only)

For the fastest possible compile times at the cost of runtime performance:

```toml
# .cargo/config.toml — append this section

# Requires nightly toolchain and:
#   rustup component add rustc-codegen-cranelift --toolchain nightly
[unstable]
codegen-backend = true

[profile.dev]
codegen-backend = "cranelift"
```

Run with: `cargo +nightly run --features dev`
