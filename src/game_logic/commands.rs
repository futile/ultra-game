use bevy::{ecs::system::SystemParam, prelude::*};

use super::{fight::FightInterface, AbilityId, AbilitySlot};
use crate::{abilities::AbilityInterface, game_logic::fight::FightStatus};

#[derive(Debug, Event)]
pub struct CastAbility {
    pub caster_e: Entity,
    pub slot_e: Option<Entity>,
    pub ability_e: Entity,
    pub fight_e: Entity,
}

#[derive(SystemParam)]
pub struct CastAbilityInterface<'w, 's> {
    ability_ids: Query<'w, 's, &'static AbilityId>,
    ability_slots: Query<'w, 's, &'static AbilitySlot>,
    ability_interface: AbilityInterface<'w, 's>,
    fight_interface: FightInterface<'w, 's>,
}

impl<'w, 's> CastAbilityInterface<'w, 's> {
    pub fn is_matching_cast(&self, cast: &CastAbility, id: &AbilityId) -> bool {
        let ability_id = self.ability_ids.get(cast.ability_e).unwrap();
        ability_id == id
    }

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

        let can_use_slot = ability.can_use_slot(slot);

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
