use bevy::prelude::*;

use super::AbilityCatalog;
use crate::game_logic::ability::{Ability, AbilityId, AbilitySlotType};

const THIS_ABILITY_ID: AbilityId = AbilityId::ChargedStrike;

fn add_to_ability_catalog(mut abilties_catalog: ResMut<AbilityCatalog>) {
    abilties_catalog.0.insert(
        THIS_ABILITY_ID,
        Ability {
            name: "Charged Strike".into(),
            id: THIS_ABILITY_ID,
            slot_type: Some(AbilitySlotType::WeaponAttack),
            #[expect(clippy::useless_format, reason = "Uniformity")]
            description: format!("Charge an extra strong strike!").into(),
        },
    );
}

#[derive(Debug)]
pub struct ChargedStrikePlugin;

impl Plugin for ChargedStrikePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, add_to_ability_catalog);
        // .add_systems(
        //     FixedUpdate,
        //     tick_needling_hex_effects.in_set(PerUpdateSet::LogicUpdate),
        // )
        // .add_systems(Update, cast_ability.in_set(PerUpdateSet::CommandResolution));
    }
}
