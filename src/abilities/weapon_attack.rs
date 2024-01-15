use bevy::prelude::*;

use super::AbilityCatalog;
use crate::{
    game_logic::{commands, Ability, AbilityId, AbilitySlot, AbilitySlotType},
    PerUpdateSet,
};

const THIS_ABILITY_ID: AbilityId = AbilityId::Attack;

fn add_to_ability_catalog(mut abilties_catalog: ResMut<AbilityCatalog>) {
    abilties_catalog.0.insert(
        AbilityId::Attack,
        Ability {
            name: "Attack".into(),
            id: THIS_ABILITY_ID,
            slot_type: AbilitySlotType::WeaponAttack,
        },
    );
}

fn cast_ability(
    mut cast_ability_events: EventReader<commands::CastAbility>,
    ability_ids: Query<&AbilityId>,
    ability_slots: Query<&AbilitySlot>,
    ability_catalog: Res<AbilityCatalog>,
) {
    for commands::CastAbility {
        caster_e,
        slot_e,
        ability_e,
    } in cast_ability_events.read()
    {
        let ability_id = ability_ids
            .get(*ability_e)
            .expect("CastAbility.ability without AbilityId");

        if *ability_id != THIS_ABILITY_ID {
            continue;
        }

        let slot = slot_e.map(|slot_e| {
            ability_slots
                .get(slot_e)
                .ok()
                .expect("CastAbility.slot without AbilitySlot")
        });

        let ability = ability_catalog
            .0
            .get(ability_id)
            .expect("CastAbility.ability without AbilityCatalog entry");

        if !ability.can_use(slot) {
            eprintln!(
                "Cannot execute commands::CastAbility due to mismatching slot: {ability:?} | Caster: {caster_e:?} | Slot: {slot_e:?} [{slot:?}]"
            );
            continue;
        }

        println!(
            "Casting ability: {ability_id:?} | Caster: {caster_e:?} | Slot: {slot_e:?} [{slot:?}]"
        );
    }
}

#[derive(Debug)]
pub struct WeaponAttackPlugin;

impl Plugin for WeaponAttackPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, add_to_ability_catalog)
            .add_systems(Update, cast_ability.in_set(PerUpdateSet::CommandResolution));
    }
}
