use bevy::{ecs::system::SystemParam, platform::collections::HashMap, prelude::*};

use crate::game_logic::ability::{Ability, AbilityId};

pub mod charged_strike;
pub mod needling_hex;
pub mod weapon_attack;

// AbilityCatalog maps AbilityId to a function that spawns the ability entity
#[derive(Resource, Default, Clone)]
pub struct AbilityCatalog(
    pub std::sync::Arc<std::sync::RwLock<HashMap<AbilityId, fn(&mut Commands) -> Entity>>>,
);

impl AbilityCatalog {
    pub fn register(&self, id: AbilityId, spawner: fn(&mut Commands) -> Entity) {
        self.0.write().unwrap().insert(id, spawner);
    }

    pub fn spawn(&self, id: AbilityId, commands: &mut Commands) -> Entity {
        let spawner = *self
            .0
            .read()
            .unwrap()
            .get(&id)
            .expect("Ability not registered");
        spawner(commands)
    }
}

#[derive(SystemParam)]
pub struct AbilityInterface<'w, 's> {
    abilities: Query<'w, 's, &'static Ability>,
}

impl<'w, 's> AbilityInterface<'w, 's> {
    pub fn get_ability_from_entity(&self, ability_e: Entity) -> &Ability {
        self.abilities
            .get(ability_e)
            .expect("ability_e without Ability component")
    }
}

#[derive(Debug)]
pub struct AbilitiesPlugin;

impl Plugin for AbilitiesPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AbilityCatalog>().add_plugins((
            weapon_attack::WeaponAttackPlugin,
            needling_hex::NeedlingHexPlugin,
            charged_strike::ChargedStrikePlugin,
        ));
    }
}
