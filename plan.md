# Unified Slot System Refactoring Plan

## Current State Analysis

**Slot=None Usage Patterns:**
- NeedlingHex: `slot_type: None` - instant cast ability that doesn't require a slot
- ChargedStrike/WeaponAttack: `slot_type: Some(AbilitySlotType::WeaponAttack)` - requires specific slot
- Current validation: `ability.slot_type == selected_slot_type` in `Ability::can_use_slot()`
- Issue: OngoingCast creation requires `slot_e.unwrap()` for slotless abilities, creating inconsistent API

**Current Architecture Problems:**
1. `HasOngoingCast` tracked on both ability entity AND slot entity (when slot exists)
2. Slotless abilities can't participate in ongoing cast interruption logic cleanly
3. API inconsistency between slotted and slotless abilities
4. Interruption logic is slot-centric but some abilities have no slot

## Proposed Solution: Universal Slot System

**Core Concept:** Every ability uses a slot, but introduce "virtual slots" for abilities that don't require physical slot resources.

### Phase 1: Add Generic Slot Types

**1.1 Extend AbilitySlotType enum**
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect)]
pub enum AbilitySlotType {
    // Physical slots (existing)
    WeaponAttack,
    ShieldDefend,
    
    // Virtual slots (new)
    InstantCast,        // For abilities like NeedlingHex
    Concentration,      // For channeled abilities without physical requirements  
    Reaction,          // For reactive abilities
}
```

**1.2 Update Ability definitions**
- Change NeedlingHex from `slot_type: None` to `slot_type: Some(AbilitySlotType::InstantCast)`
- All abilities must specify a slot type

**1.3 Create virtual slot entities**
- Spawn virtual slot entities for each character (InstantCast, Concentration, etc.)
- Use same `Holds<AbilitySlot>/Held<AbilitySlot>` relationship pattern
- Virtual slots allow unlimited concurrent usage (no resource constraints)

### Phase 2: Refactor Ongoing Cast System

**2.1 Simplify HasOngoingCast tracking**
- Remove `HasOngoingCast` from ability entities entirely
- Only track `HasOngoingCast` on slot entities (both physical and virtual)
- Ongoing cast interruption becomes uniform: new cast on slot interrupts existing cast

**2.2 Update OngoingCast creation**
- Remove `slot_e.unwrap()` - all abilities now have guaranteed slot
- Simplify API: `UseAbility.slot_e` becomes non-optional `Entity`

**2.3 Implement slot-specific interruption rules**
```rust
pub enum SlotInterruptionBehavior {
    Replace,      // New cast replaces existing (physical slots)
    Queue,        // New cast waits for existing to finish  
    Concurrent,   // Multiple casts allowed (virtual slots like InstantCast)
}
```

### Phase 3: Create SlotInterface (Revised Approach)

**3.1 SlotCastingInterface SystemParam**
```rust
#[derive(SystemParam)]
pub struct SlotCastingInterface<'w, 's> {
    slots: Query<'w, 's, &'static AbilitySlot>,
    ongoing_casts: Query<'w, 's, &'static HasOngoingCast>,
    ongoing_cast_interface: OngoingCastInterface<'w, 's>,
}

impl SlotCastingInterface {
    pub fn can_cast_on_slot(&self, slot_e: Entity, ability_slot_type: AbilitySlotType) -> bool
    pub fn start_cast_on_slot(&mut self, slot_e: Entity, cast: OngoingCast) -> Result<Entity, CastError>
    pub fn interrupt_cast_on_slot(&mut self, slot_e: Entity)
}
```

**3.2 Move free functions into interface**
- Consolidate slot-related logic from various ability implementations
- Centralized cast validation and interruption logic
- Clean separation of concerns

### Phase 4: Update Ability Implementations

**4.1 Refactor ability casting systems**
- Update all ability implementations to use `SlotCastingInterface`
- Remove direct `OngoingCastInterface` usage from abilities
- Standardize casting pattern across all abilities

**4.2 Update UI and command systems**
- Modify fight UI to handle virtual slots (may not need visual representation)
- Update `UseAbility` command validation
- Ensure backwards compatibility for existing gameplay

### Phase 5: Advanced Features (Optional)

**5.1 Slot resource management**
- Add cooldowns to virtual slots if needed
- Implement slot-specific constraints (e.g., max concurrent InstantCast abilities)

**5.2 Enhanced interruption system**
- Priority-based interruption for different ability types
- Slot-specific interruption animations/effects

## Implementation Order

1. **Phase 1.1-1.2**: Extend slot types, update ability definitions (breaking change - coordinate carefully)
2. **Phase 1.3**: Create virtual slot spawning system
3. **Phase 2.1-2.2**: Refactor OngoingCast system (major refactor)
4. **Phase 3.1**: Implement SlotCastingInterface
5. **Phase 4.1**: Update ability implementations one by one
6. **Phase 4.2**: Update UI and commands
7. **Phase 2.3**: Add advanced interruption behaviors
8. **Phase 5**: Optional enhancements

## Testing Strategy

- **Unit tests**: SlotCastingInterface behavior
- **Integration tests**: Cast interruption scenarios across all slot types
- **Manual testing**: Verify UI still works correctly
- **Performance testing**: Ensure virtual slots don't impact performance

## Benefits

1. **Unified API**: All abilities use same casting interface
2. **Clear interruption logic**: Slot-based interruption rules
3. **Extensible**: Easy to add new slot types and behaviors
4. **Clean architecture**: Single responsibility for each component
5. **Backwards compatible**: Existing gameplay mechanics preserved

## Risks

- **Large refactor**: Multiple systems need coordinated changes
- **Breaking changes**: Ability definitions change format
- **Complexity**: More slot entities in the world
- **Testing burden**: Need comprehensive test coverage for new patterns

## Migration Notes

- Keep old ability definitions working during transition
- Implement feature flags for gradual rollout
- Provide clear migration guide for any external ability definitions