use bevy::prelude::*;

use super::{Ability, AbilityId, AbilitySlot};

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

pub struct CommandsPlugin;

impl Plugin for CommandsPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<CastAbility>();
    }
}
