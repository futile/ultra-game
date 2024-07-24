use bevy::{ecs::system::SystemParam, prelude::*, utils::HashMap};

use crate::game_logic::ability::{Ability, AbilityId};

pub mod needling_hex;
pub mod weapon_attack;

#[derive(Debug, Resource, Reflect, Default)]
#[reflect(Resource)]
pub struct AbilityCatalog(pub HashMap<AbilityId, Ability>);

#[derive(SystemParam)]
pub struct AbilityInterface<'w, 's> {
    ability_ids: Query<'w, 's, &'static AbilityId>,
    ability_catalog: Res<'w, AbilityCatalog>,
}

impl<'w, 's> AbilityInterface<'w, 's> {
    pub fn get_ability_from_entity(&self, ability_e: Entity) -> &Ability {
        let ability_id = self
            .ability_ids
            .get(ability_e)
            .expect("ability_e without AbilityId");

        let ability = self
            .ability_catalog
            .0
            .get(ability_id)
            .unwrap_or_else(|| panic!("AbilityId `{:?}` not in catalog", ability_id));

        ability
    }
}

#[derive(Debug)]
pub struct AbilitiesPlugin;

impl Plugin for AbilitiesPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AbilityCatalog>()
            .register_type::<AbilityCatalog>()
            .add_plugins((
                weapon_attack::WeaponAttackPlugin,
                needling_hex::NeedlingHexPlugin,
            ));
    }
}
