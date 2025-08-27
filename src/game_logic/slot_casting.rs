use bevy::{ecs::system::SystemParam, prelude::*};

use super::{
    ability_slots::{AbilitySlot, AbilitySlotType},
    ongoing_cast::{OngoingCast, OngoingCastInterface},
};

#[derive(SystemParam)]
pub struct SlotCastingInterface<'w, 's> {
    slots: Query<'w, 's, &'static AbilitySlot>,
    ongoing_cast_interface: OngoingCastInterface<'w, 's>,
}

impl<'w, 's> SlotCastingInterface<'w, 's> {
    /// Checks if an ability with the given slot type can be cast on the specified slot
    pub fn can_cast_on_slot(&self, slot_e: Entity, ability_slot_type: AbilitySlotType) -> bool {
        let Ok(slot) = self.slots.get(slot_e) else {
            return false;
        };
        
        slot.tpe == ability_slot_type
    }

    /// Starts a cast, automatically interrupting any existing cast on the same slot
    pub fn start_cast(&mut self, cast: OngoingCast) -> Entity {
        // The OngoingCast system will automatically handle interruption when we create the new cast
        // (see on_add_ongoing_cast observer in ongoing_cast.rs)
        self.ongoing_cast_interface.start_new_cast(cast)
    }

    /// Interrupts any ongoing cast on the specified slot
    pub fn interrupt_cast_on_slot(&mut self, slot_e: Entity) {
        self.ongoing_cast_interface.cancel_ongoing_cast(slot_e);
    }
}