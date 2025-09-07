# Slot Cooldowns

This document describes the slot-defined cooldown system in the ultra-game project.

## Overview

Slots automatically apply their own cooldowns when abilities use them, centralizing slot cooldown logic across the codebase.

## Core Architecture

### Data Structure

**File**: `src/game_logic/ability_slots.rs`

```rust
#[derive(Debug, Component, Reflect)]
pub struct AbilitySlot {
    pub tpe: AbilitySlotType,
    pub on_use_cooldown: Option<Duration>,
}
```

- `on_use_cooldown: Option<Duration>` - If `Some(duration)`, a cooldown of that duration is applied to the slot when used
- `None` means no slot cooldown is applied

### Automatic Application

#### Instant Abilities

**File**: `src/game_logic/ability_casting.rs`

The `AbilityCastingInterface::use_slot()` method automatically applies slot cooldowns for instant abilities:

```rust
pub fn use_slot(&mut self, slot_e: Entity) {
    self.interrupt_cast_on_slot(slot_e);
    
    // Apply slot-defined cooldown if present
    if let Ok(slot) = self.ability_slots.get(slot_e)
        && let Some(cooldown_duration) = slot.on_use_cooldown {
            self.commands
                .entity(slot_e)
                .insert(Cooldown::new(cooldown_duration));
        }
}
```

#### Cast-Time Abilities

**File**: `src/game_logic/ability_casting.rs`

An observer handles slot cooldown application when cast-time abilities finish successfully:

```rust
pub fn apply_slot_cooldown_on_cast_finish(
    trigger: Trigger<OngoingCastFinishedSuccessfully>,
    ongoing_casts: Query<&OngoingCast>,
    ability_slots: Query<&AbilitySlot>,
    mut commands: Commands,
) {
    let ongoing_cast = ongoing_casts.get(trigger.target()).unwrap();
    let slot_e = ongoing_cast.slot_e;
    
    // Apply slot-defined cooldown if present
    if let Ok(slot) = ability_slots.get(slot_e)
        && let Some(cooldown_duration) = slot.on_use_cooldown {
            commands
                .entity(slot_e)
                .insert(Cooldown::new(cooldown_duration));
        }
}
```

This observer is registered in `AbilityCastingPlugin` and automatically triggered when casts complete.

## Current Configuration

**File**: `src/main.rs`

Slot cooldowns are configured when slots are created:

- **WeaponAttack slots**: `Some(Duration::from_secs(1))` - 1 second cooldown
- **Magic slots**: `Some(Duration::from_secs(2))` - 2 second cooldown  
- **ShieldDefend slots**: `None` - No cooldown (defensive abilities)

## Cooldown Types

The system supports two independent cooldown types:

1. **Ability Cooldowns**: Applied to the ability entity (`ability_e`)
   - Prevents the specific ability from being used
   - Duration varies per ability (5s for Attack, 20s for ChargedStrike, 30s for NeedlingHex)

2. **Slot Cooldowns**: Applied to the slot entity (`slot_e`)
   - Prevents any ability from using that slot
   - Duration defined by the slot's `on_use_cooldown` field
   - Automatically applied via `use_slot()` or when casts finish

Both cooldowns are independent and both must be ready for an ability to be castable.

## Key Files

- **`src/game_logic/ability_slots.rs`**: Defines `AbilitySlot` with `on_use_cooldown` field
- **`src/game_logic/ability_casting.rs`**: Implements automatic slot cooldown application, observer, and plugin
- **`src/game_logic/cooldown.rs`**: Core cooldown system
- **`src/main.rs`**: Slot creation with cooldown configuration
- **`src/abilities/*.rs`**: Individual abilities use the unified interface

## Implementation Notes

- Slot cooldowns use the same `Cooldown` component as ability cooldowns
- Cooldown validation in `is_valid_cast()` checks both ability and slot entities for cooldowns
- The system works automatically for both instant abilities (via `use_slot()`) and cast-time abilities (via observer)
- Observer-based architecture ensures slot cooldowns are applied consistently when casts complete