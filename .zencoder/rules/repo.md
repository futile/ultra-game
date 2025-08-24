---
description: Repository Information Overview
alwaysApply: true
---

# Ultra Game Information

## Summary
A Rust-based game project built with the Bevy game engine. The game appears to be a turn-based combat game with abilities, health systems, and faction-based gameplay mechanics.

## Structure
- **src/**: Main source code directory containing game logic, UI, and abilities
  - **abilities/**: Contains specific game abilities like charged_strike, needling_hex, and weapon_attack
  - **fight_ui/**: UI components for the combat system
  - **game_logic/**: Core game mechanics including health, factions, and combat resolution
  - **utils/**: Utility functions and helpers
- **assets/**: Game assets directory
- **support/**: Support scripts and tools
- **docs/**: Documentation files

## Language & Runtime
**Language**: Rust
**Version**: Nightly (nightly-2025-06-03)
**Build System**: Cargo with Nix/Earthly support
**Package Manager**: Cargo

## Dependencies
**Main Dependencies**:
- bevy (0.16.1) - Game engine with wayland and dynamic_linking features
- bevy-inspector-egui (0.31.0) - Debug inspection tools
- ahash (0.8.12) - Hashing library
- derive_more (2.0.1) - Derive macro extensions
- itertools (0.14.0) - Iterator utilities

**Development Dependencies**:
- mold-wrapped - Linker for faster compilation
- clang_16 - C/C++ compiler
- Various system libraries for Bevy (udev, alsa-lib, vulkan-loader, etc.)

## Build & Installation
```bash
# Regular build
cargo build

# Run with X11 on Wayland
just run-x11-on-wayland

# Build with timing information
just build-with-timings

# Build with Earthly
earthly +build
```

## Nix Development Environment
The project uses Nix flakes for reproducible development environments:

```bash
# Enter development shell
nix develop

# Build with Nix
nix build
```

## Project Structure
The game is structured around a turn-based combat system with:

1. **Ability System**: Modular abilities that characters can use in combat
   - Located in `src/abilities/`
   - Examples: ChargedStrike, NeedlingHex, WeaponAttack

2. **Combat Logic**: 
   - Located in `src/game_logic/`
   - Handles damage resolution, health tracking, and combat state

3. **UI Components**:
   - Located in `src/fight_ui/`
   - Renders the combat interface and effects

4. **Main Game Loop**:
   - Defined in `src/main.rs`
   - Uses Bevy's ECS (Entity Component System) architecture
   - Organized with SystemSets for update ordering

## Testing
**Framework**: Rust's built-in testing framework
**Test Location**: Tests are embedded within source files
**Naming Convention**: Standard Rust test modules with `#[cfg(test)]`
**Run Command**:
```bash
cargo test
```