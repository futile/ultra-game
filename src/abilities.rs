use bevy::{prelude::*, utils::HashMap};

use crate::core_logic::{Ability, AbilityId};

mod weapon_attack;

#[derive(Debug, Resource, Reflect, Default)]
#[reflect(Resource)]
pub struct AbilityCatalog(pub HashMap<AbilityId, Ability>);

#[derive(Debug)]
pub struct AbilitiesPlugin;

impl Plugin for AbilitiesPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AbilityCatalog>()
            .register_type::<AbilityCatalog>()
            .add_plugins(weapon_attack::WeaponAttackPlugin);
    }
}
