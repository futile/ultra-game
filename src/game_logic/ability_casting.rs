use bevy::{ecs::system::SystemParam, prelude::*};

use super::{
    ability::AbilityId,
    ability_slots::AbilitySlot,
    fight::{FightInterface, FightStatus},
    ongoing_cast::{OngoingCast, OngoingCastInterface},
};
use crate::abilities::AbilityInterface;

#[derive(SystemParam)]
pub struct AbilityCastingInterface<'w, 's> {
    ability_ids: Query<'w, 's, &'static AbilityId>,
    ability_slots: Query<'w, 's, &'static AbilitySlot>,
    ability_interface: AbilityInterface<'w, 's>,
    fight_interface: FightInterface<'w, 's>,
    ongoing_cast_interface: OngoingCastInterface<'w, 's>,
}

/// Represents the usage of an ability
#[derive(Debug, Clone)]
pub struct UseAbility {
    pub caster_e: Entity,
    pub slot_e: Entity,
    pub ability_e: Entity,
    pub fight_e: Entity,
}

impl<'w, 's> AbilityCastingInterface<'w, 's> {
    /// Checks if the cast request matches the given ability ID
    pub fn is_matching_cast(&self, cast: &UseAbility, id: &AbilityId) -> bool {
        let ability_id = self.ability_ids.get(cast.ability_e).unwrap();
        ability_id == id
    }

    /// Validates if the cast request is valid (fight ongoing, slot compatibility)
    pub fn is_valid_cast(&self, cast: &UseAbility) -> bool {
        match self.fight_interface.get_fight_status(cast.fight_e) {
            FightStatus::Ongoing => (),
            FightStatus::Ended => return false,
        };

        let ability = self
            .ability_interface
            .get_ability_from_entity(cast.ability_e);
        let slot = self.ability_slots.get(cast.slot_e).unwrap();

        ability.can_use_slot(slot)
    }

    // TODO: probably merge `use_slot()` and `start_cast()` into an fn like `use_ability()`,
    // which takes an `(&)UseAbility` and resolves the casting logic. Or this might be exactly
    // backwards, because ability impls (those triggered by `UseAbility`) call this, and we don't
    // want to loop.

    /// Uses a slot for an instant ability, interrupting any ongoing cast on it
    pub fn use_slot(&mut self, slot_e: Entity) {
        self.interrupt_cast_on_slot(slot_e);
    }

    /// Starts a cast on a slot, automatically interrupting any existing cast on the same slot
    pub fn start_cast(&mut self, cast: OngoingCast) -> Entity {
        // The OngoingCast system will automatically handle interruption when we create the new cast
        // (see on_add_ongoing_cast observer in ongoing_cast.rs)
        self.ongoing_cast_interface.start_new_cast(cast)
    }

    /// Interrupts any ongoing cast on the specified slot (low-level method)
    fn interrupt_cast_on_slot(&mut self, slot_e: Entity) {
        self.ongoing_cast_interface.cancel_ongoing_cast(slot_e);
    }
}
