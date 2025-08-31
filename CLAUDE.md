# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Development Commands

- **Build**: `cargo build` (standard build), `cargo build --release` (optimized build)
- **Run**: `cargo run` (standard), `just run-x11-on-wayland` (for Wayland systems)
- **Test**: `cargo test` (run all tests), `cargo test <test_name>` (run specific test)
- **Format**: `cargo fmt` (uses rustfmt.toml config with 100 char width, requires nightly Rust)
- **Lint**: `cargo check` and `cargo clippy` (standard linting)
- **Just recipes**: Run `just` to see all available commands in justfile

## Architecture Overview

This is a **Bevy-based game engine project** implementing a real-time, but pauseable, combat system.
The architecture follows Bevy's Entity-Component-System (ECS) pattern with a modular plugin structure.

### Core Systems Architecture

- **Entity Relationships**: Uses custom `Holds/Held<T>` pattern instead of standard Bevy hierarchies for flexible entity relationships
- **System Ordering**: Defined execution order via `PerUpdateSet` enum with phases (at the time of writing): CommandResolution → TimeUpdate → LogicUpdate → DamageResolution → FightEndChecking
- **Dual-Schedule Design**: FixedUpdate for game logic, Update for UI and input handling

### Module Structure

- `game_logic/`: Core combat mechanics
  - `ability.rs`: Ability definitions and casting system
  - `ability_casting.rs`: Unified interface for ability validation and execution
  - `commands.rs`: Command pattern for game actions
  - `effects.rs`: Timed effects system with `FiniteRepeatingTimer`
  - `fight.rs`: Combat encounter management
  - `damage_resolution.rs`: Damage calculation and application
  - `ongoing_cast.rs`: Cast timing and interruption system
  - `cooldown.rs`: Cooldown interface and management
- `abilities/`: Specific ability implementations (WeaponAttack, NeedlingHex, ChargedStrike)
- `fight_ui/`: UI rendering and interaction systems
- `utils/`: Shared utilities including timing and relationship systems

### Key Design Patterns

- **Command Pattern**: All game actions go through `game_logic::commands` for validation and execution
- **Effect System**: Time-based effects using components with tick-based resolution
- **Unified Slot System**: Every ability requires a slot to cast - slot types include WeaponAttack, ShieldDefend, and Magic
- **Casting System**: Abilities support cast times with automatic interruption mechanics
- **Relationship Tracking**: Custom `Holds<T>/Held<T>` components for entity relationships instead of Bevy's parent/child system
- **Interface Pattern**: Many plugins provide `Interface`-named SystemParam types that consolidate related functionality (e.g., `AbilityCastingInterface` combines validation + execution)

### Ability Casting & Interruption Mechanics

**Slot-Based Casting:**

- All abilities require a slot to cast (no slot=None abilities)
- Slot types: `WeaponAttack`, `ShieldDefend`, `Magic`
- All slots work identically for interruption purposes

**Interruption Rules:**

- **Any ability usage interrupts ongoing casts on the same slot**
- Instant abilities (WeaponAttack, NeedlingHex): Call `ability_casting_interface.use_slot(slot_e)` before execution
- Cast abilities (ChargedStrike): Call `ability_casting_interface.start_cast()` - interruption handled automatically
- Interruption occurs via observers in `ongoing_cast.rs` when new casts are created

**AbilityCastingInterface Usage:**

- **Validation**: `is_valid_cast()`, `is_matching_cast()`, `can_cast_on_slot()`
- **Execution**: `use_slot(slot_e)` for instant abilities, `start_cast(OngoingCast)` for cast abilities
- **Manual**: `interrupt_cast_on_slot(slot_e)` for special cases
- Replaces old separate `CastAbilityInterface` + `SlotCastingInterface` pattern

### Development Notes

- Requires **nightly Rust** (see rust-toolchain.toml) for rustfmt unstable features
- Uses Bevy 0.16.1 with dynamic linking for faster compilation
- Performance optimizations in Cargo.toml for debug builds (opt-level = 1 for main code, 3 for dependencies)
- Entity inspector available via bevy-inspector-egui for debugging
