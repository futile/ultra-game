# Slot Cooldowns

This document describes the slot-defined cooldown system implemented in the ultra-game project.

## Overview

Slots can now define their own cooldowns that are automatically applied when abilities use them. This centralizes slot cooldown logic and eliminates duplicate code across ability implementations.

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

**File**: `src/game_logic/ability_casting.rs`

The `AbilityCastingInterface::use_slot()` method automatically applies slot cooldowns:

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

All abilities that call `use_slot()` automatically get slot cooldowns applied.

## Current Configuration

**File**: `src/main.rs`

Slot cooldowns are configured when slots are created:

- **WeaponAttack slots**: `Some(Duration::from_secs(1))` - 1 second cooldown
- **Magic slots**: `Some(Duration::from_secs(2))` - 2 second cooldown  
- **ShieldDefend slots**: `None` - No cooldown (defensive abilities)

## Ability Integration

### Instant Abilities

**Files**: 
- `src/abilities/weapon_attack.rs`
- `src/abilities/needling_hex.rs`

Instant abilities call `ability_casting_interface.use_slot(slot_e)` which automatically applies the slot cooldown.

**Previous manual slot cooldown code has been removed** from these abilities. They now only apply ability-specific cooldowns:

```rust
// OLD (removed):
commands.entity(*slot_e).insert(Cooldown::new(THIS_ABILITY_SLOT_COOLDOWN));

// NEW (automatic via use_slot()):
ability_casting_interface.use_slot(*slot_e);
```

### Cast-Time Abilities

**File**: `src/abilities/charged_strike.rs`

Cast-time abilities call `ability_casting_interface.start_cast()` instead of `use_slot()`.

**Current Status**: Slot cooldowns are NOT yet applied when cast-time abilities finish. This is planned for future implementation via:
- Event when OngoingCast finishes successfully
- Logic to apply slot cooldown at that time (in ability_casting.rs, not ongoing_cast.rs)

## Cooldown Types

The system now supports two independent cooldown types:

1. **Ability Cooldowns**: Applied to the ability entity (`ability_e`)
   - Prevents the specific ability from being used
   - Duration varies per ability (5s for Attack, 20s for ChargedStrike, 30s for NeedlingHex)

2. **Slot Cooldowns**: Applied to the slot entity (`slot_e`)
   - Prevents any ability from using that slot
   - Duration defined by the slot's `on_use_cooldown` field
   - Automatically applied via `use_slot()`

Both cooldowns are independent and both must be ready for an ability to be castable.

## Key Files

- **`src/game_logic/ability_slots.rs`**: Defines `AbilitySlot` with `on_use_cooldown` field
- **`src/game_logic/ability_casting.rs`**: Implements automatic slot cooldown application in `use_slot()`
- **`src/game_logic/cooldown.rs`**: Core cooldown system (unchanged)
- **`src/main.rs`**: Slot creation with cooldown configuration
- **`src/abilities/*.rs`**: Individual abilities (manual slot cooldown code removed)

## Implementation Notes

- Slot cooldowns use the same `Cooldown` component as ability cooldowns
- Cooldown validation in `is_valid_cast()` checks both ability and slot entities for cooldowns
- The system preserves all existing behavior while centralizing slot cooldown logic
- No changes needed to the core cooldown system - it already supported cooldowns on any entity