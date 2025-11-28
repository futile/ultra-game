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
- **Dual-Schedule Design**: FixedUpdate for game logic, Update for UI and input handling, as well as AI calculation.

### Module Structure

- `game_logic/`: Core combat mechanics
  - `ability.rs`: Ability components (`Ability`, `AbilitySlotRequirement`, `AbilityCooldown`, `AbilityCastTime`) and `PerformAbility` event
  - `ability_casting.rs`: Validation systems (veto pattern with `CastRequest` entities), `AbilityCastingInterface`, and observers
  - `ai_behavior.rs`: AI behavior, uses the `big-brain` crate for bevy. See `docs/big-brain.md` in this repo for a summary.
  - `commands.rs`: Command pattern for game actions (`GameCommand` with `UseAbility`)
  - `effects.rs`: Timed effects system with `FiniteRepeatingTimer`
  - `fight.rs`: Combat encounter management
  - `damage_resolution.rs`: Damage calculation and application
  - `ongoing_cast.rs`: Cast timing system - all abilities go through `OngoingCast` → `PerformAbility` flow
  - `cooldown.rs`: Cooldown interface and management
    - The file `docs/slot-cooldowns.md` contains detailed information on cooldowns for slots.
- `abilities/`: Specific ability implementations as observers responding to `PerformAbility` events (WeaponAttack, NeedlingHex, ChargedStrike)
- `fight_ui/`: UI rendering and interaction systems
- `utils/`: Shared utilities including timing and relationship systems

Many modules use the `src/foo.rs` file instead of `src/foo/mod.rs`.

### Key Design Patterns

- **Command Pattern**: All game actions go through `game_logic::commands` for validation and execution
- **Observer Pattern**: Abilities use Bevy Observers to respond to `PerformAbility` events, decoupling execution from casting mechanics
- **Component-Based Abilities**: Abilities are entities with components defining their properties (cooldowns, cast times, slot requirements)
- **Veto Validation Pattern**: Validation uses `CastRequest` entities where systems add `CastFailed<Reason>` components if checks fail
- **Effect System**: Time-based effects using components with tick-based resolution
- **Unified Slot System**: Every ability requires a slot to cast - slot types include WeaponAttack, ShieldDefend, and Magic
- **Unified Casting Flow**: All abilities (instant and casted) go through `OngoingCast` → `PerformAbility` flow
- **Relationship Tracking**: Custom `Holds<T>/Held<T>` components for entity relationships instead of Bevy's parent/child system
- **Interface Pattern**: Many plugins provide `Interface`-named SystemParam types that consolidate related functionality (e.g., `AbilityCastingInterface` for validation)

### Ability System Architecture

**Component-Based Abilities:**

- Abilities are **entities** with components defining their properties:
  - `Ability` - name and description
  - `AbilitySlotRequirement` - which slot type the ability requires
  - `AbilityCooldown` - how long until the ability can be cast again
  - `AbilityCastTime` - cast duration (Duration::ZERO for instant abilities)
- Abilities are spawned via `AbilityCatalog` and associated with casters via `Held<Ability>` component

**Validation Flow (Veto Pattern):**

- Ability usage requests create `CastRequest` entities (spawned from `GameCommand`)
- Validation systems add `CastFailed<Reason>` components if checks fail:
  - `check_ability_cooldowns` - checks if ability is on cooldown
  - `check_slot_cooldowns_real` - checks if slot is on cooldown  
  - `check_slot_requirements` - checks if ability matches slot type
- Valid requests (without `CastFailed`) are processed into `OngoingCast` entities
- Failed requests are automatically despawned

**Execution Flow:**

- All abilities (instant and casted) go through `OngoingCast` → `PerformAbility` flow
- `OngoingCast` component tracks cast progress with timer
- When cast completes, `OngoingCastFinishedSuccessfully` event is triggered
- `trigger_perform_ability` observer converts this to `PerformAbility` event
- Specific ability logic is implemented as observers responding to `PerformAbility`

**Interruption Rules:**

- **Any ability usage interrupts ongoing casts on the same slot**
- Interruption occurs via observers in `ongoing_cast.rs` when new casts are created
- Both instant and cast-time abilities trigger interruption automatically

**Cooldown Mechanics:**

- **Ability Cooldown**: How long until THIS specific ability can be cast again (component on ability entity)
- **Slot Cooldown**: How long until ANY ability can be cast on this slot (component on slot entity)
- After casting an ability, both cooldowns apply independently:
  - The slot is blocked for the slot cooldown duration (prevents any ability on that slot)
  - The specific ability cannot be cast again until its ability cooldown expires
  - Example: WeaponAttack (5s ability cooldown, 1s slot cooldown) → slot available after 1s for other WeaponAttack slot abilities, but WeaponAttack itself needs 5s
- Slot cooldowns are applied automatically via observer (`apply_slot_cooldown_on_cast_finish`)

**AbilityCastingInterface Usage:**

- **Validation**: `is_valid_cast()` - checks cooldowns and slot requirements using components
- **Client-Side Prediction**: UI uses `is_valid_cast()` to predict validity before user input
- **AI Integration**: AI queries ability components and uses `is_valid_cast()` for decision-making

### Development Notes

- Requires **nightly Rust** (see rust-toolchain.toml) for rustfmt unstable features
- Uses Bevy 0.16.1 with dynamic linking for faster compilation
- Performance optimizations in Cargo.toml for debug builds (opt-level = 1 for main code, 3 for dependencies)
- Entity inspector available via bevy-inspector-egui for debugging
- After finishing a task, when `cargo check`, `cargo clippy`, etc. pass, always format with `cargo fmt`

### Style Guidelines

#### Doc-Comments

- Always add doc-comments for everything you create, and update doc-comments when you change something, if it's relevant for the doc comments.
- For functions, document parameters, return value, possible errors and panic reasons, unless they aren't really important.
- Document in a "black-box" style, i.e., what's relevant from outside, don't describe internal behavior of the function that could change (which would be "white-box" style commenting).
- Try to explain "why" things exist, in addition to describing "what" exists (which is always already described by the thing, function, etc. itself).

#### Copying code from documentation or examples

- When copying code from some documentation or some example (e.g., documentation/example of a dependency), ONLY change what's required for the current task
- **Don't** make other on-the-fly changes to code you copy, or, if really necessary, mention it & explain why. It is ok if you are wrong, the user simply wants to understand why.
