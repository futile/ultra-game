use std::time::Duration;

use bevy::prelude::*;

use super::AbilityCatalog;
use crate::core_logic::{Ability, AbilityId, AbilitySlotType};

fn add_to_ability_catalog(mut abilties_catalog: ResMut<AbilityCatalog>) {
    abilties_catalog.0.insert(
        AbilityId::Attack,
        Ability {
            name: "Attack".into(),
            id: AbilityId::Attack,
            slot_type: AbilitySlotType::WeaponAttack,
            cooldown: Duration::from_secs_f32(1.0),
        },
    );
}

#[derive(Debug)]
pub struct WeaponAttackPlugin;

impl Plugin for WeaponAttackPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, add_to_ability_catalog);
    }
}
