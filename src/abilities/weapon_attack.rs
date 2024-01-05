use bevy::prelude::*;

use super::AbilityCatalog;
use crate::{
    game_logic::{commands, Ability, AbilityId, AbilitySlotType},
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
) {
    for cast_ability in cast_ability_events.read() {
        let ability_id = ability_ids
            .get(cast_ability.ability)
            .expect("CastAbility.ability without AbilityId");

        if *ability_id != THIS_ABILITY_ID {
            continue;
        }

        println!("Casting ability: {:?}", ability_id);
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
