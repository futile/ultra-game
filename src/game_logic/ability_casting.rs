use bevy::{ecs::system::SystemParam, prelude::*};
use derive_more::{Display, Error};

use super::{
    ability::AbilityId,
    ability_slots::AbilitySlot,
    fight::{FightInterface, FightStatus},
    ongoing_cast::{OngoingCast, OngoingCastFinishedSuccessfully, OngoingCastInterface},
};
use crate::{abilities::AbilityInterface, game_logic::cooldown::Cooldown};

#[derive(SystemParam)]
pub struct AbilityCastingInterface<'w, 's> {
    ability_ids: Query<'w, 's, &'static AbilityId>,
    ability_slots: Query<'w, 's, &'static AbilitySlot>,
    has_cooldown: Query<'w, 's, Has<Cooldown>>,
    pub ability_interface: AbilityInterface<'w, 's>,
    pub fight_interface: FightInterface<'w, 's>,
    pub ongoing_cast_interface: OngoingCastInterface<'w, 's>,
    commands: Commands<'w, 's>,
}

/// Represents the usage of an ability
#[derive(Debug, Clone)]
pub struct UseAbility {
    pub caster_e: Entity,
    pub slot_e: Entity,
    pub ability_e: Entity,
    pub fight_e: Entity,
}

#[derive(Debug, Display, Error)]
pub enum InvalidCastReason {
    FightEnded,
    AbilityOrSlotOnCooldown,
    CantUseSlot,
}

impl<'w, 's> AbilityCastingInterface<'w, 's> {
    /// Checks if the cast request matches the given ability ID
    pub fn is_matching_cast(&self, cast: &UseAbility, id: &AbilityId) -> bool {
        let ability_id = self.ability_ids.get(cast.ability_e).unwrap();
        ability_id == id
    }

    /// Validates if the cast request is valid (fight ongoing, slot compatibility)
    pub fn is_valid_cast(&self, cast: &UseAbility) -> Result<(), InvalidCastReason> {
        match self.fight_interface.get_fight_status(cast.fight_e) {
            FightStatus::Ongoing => (),
            FightStatus::Ended => {
                return Err(InvalidCastReason::FightEnded);
            }
        };

        let ability = self
            .ability_interface
            .get_ability_from_entity(cast.ability_e);

        if self
            .has_cooldown
            .iter_many([cast.ability_e, cast.slot_e])
            .any(|has_cd| has_cd)
        {
            return Err(InvalidCastReason::AbilityOrSlotOnCooldown);
        }

        let slot = self.ability_slots.get(cast.slot_e).unwrap();

        if !ability.can_use_slot(slot) {
            return Err(InvalidCastReason::CantUseSlot);
        }

        Ok(())
    }

    // TODO: probably merge `use_slot()` and `start_cast()` into an fn like `use_ability()`,
    // which takes an `(&)UseAbility` and resolves the casting logic. Or this might be exactly
    // backwards, because ability impls (those triggered by `UseAbility`) call this, and we don't
    // want to loop.

    /// Uses a slot for an instant ability, interrupting any ongoing cast on it
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

/// Observer that applies slot cooldowns when ongoing casts finish successfully
fn apply_slot_cooldown_on_cast_finish(
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

#[derive(Debug)]
pub struct AbilityCastingPlugin;

impl Plugin for AbilityCastingPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(apply_slot_cooldown_on_cast_finish);
    }
}
