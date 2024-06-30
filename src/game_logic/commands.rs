use bevy::{ecs::system::SystemParam, prelude::*};

use super::{fight::FightInterface, Ability, AbilityId, AbilitySlot};
use crate::{abilities::AbilityInterface, game_logic::fight::FightStatus};

#[derive(Debug, Event)]
pub struct CastAbility {
    pub caster_e: Entity,
    pub slot_e: Option<Entity>,
    pub ability_e: Entity,
    pub fight_e: Entity,
}

impl CastAbility {
    pub fn is_valid_matching_ability_cast(
        &self,
        ability: &Ability,
        ability_ids: &Query<&AbilityId>,
        ability_slots: &Query<&AbilitySlot>,
    ) -> bool {
        let matching_id = {
            let ability_id = ability_ids.get(self.ability_e).unwrap();
            ability_id == &ability.id
        };

        if !matching_id {
            return false;
        }

        let slot: Option<&AbilitySlot> =
            self.slot_e.map(|slot_e| ability_slots.get(slot_e).unwrap());

        let can_use = ability.can_use(slot);

        if !can_use {
            eprintln!("Cannot execute commands::CastAbility due to mismatching slot: {self:?} | SlotType: {slot:?}");
            return false;
        }

        true
    }
}

#[derive(SystemParam)]
pub struct CastAbilityInterface<'w, 's> {
    ability_slots: Query<'w, 's, &'static AbilitySlot>,
    ability_interface: AbilityInterface<'w, 's>,
    fight_interface: FightInterface<'w, 's>,
}

impl<'w, 's> CastAbilityInterface<'w, 's> {
    pub fn is_valid_cast(&self, cast: &CastAbility) -> bool {
        match self.fight_interface.get_fight_status(cast.fight_e) {
            FightStatus::Ongoing => (),
            FightStatus::Ended => return false,
        };

        let ability = self
            .ability_interface
            .get_ability_from_entity(cast.ability_e);
        let slot: Option<&AbilitySlot> = cast
            .slot_e
            .map(|slot_e| self.ability_slots.get(slot_e).unwrap());

        let can_use_slot = ability.can_use(slot);

        #[expect(clippy::let_and_return, reason = "better readability")]
        can_use_slot
    }
}

pub struct CommandsPlugin;

impl Plugin for CommandsPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<CastAbility>();
    }
}
