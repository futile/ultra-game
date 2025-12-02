use bevy::{ecs::system::SystemParam, platform::collections::HashMap, prelude::*};

use crate::game_logic::ability::{Ability, AbilityId};

pub mod charged_strike;
pub mod needling_hex;
pub mod weapon_attack;

pub type AbilitySpawner = fn(&mut Commands) -> Entity;

// AbilityCatalog maps AbilityId to a function that spawns the ability entity
#[derive(Resource, Default, Clone)]
pub struct AbilityCatalog(
    pub std::sync::Arc<std::sync::RwLock<HashMap<AbilityId, AbilitySpawner>>>,
);

impl AbilityCatalog {
    pub fn register(&self, id: AbilityId, spawner: AbilitySpawner) {
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

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::AbilityCatalog;
    use crate::{
        abilities::{
            needling_hex::{NeedlingHexAbility, NeedlingHexEffect, NeedlingHexPlugin},
            weapon_attack::WeaponAttackPlugin,
        },
        game_logic::{
            ability::{Ability, AbilityId},
            ability_casting::AbilityCastingPlugin,
            commands::CommandsPlugin,
            damage_resolution::{DamageResolutionPlugin, DealDamage},
            effects::HasEffects,
            fight::FightPlugin,
            ongoing_cast::{OngoingCastFinishedSuccessfully, OngoingCastPlugin},
        },
        test_utils::{TestFightEntities, spawn_test_fight},
    };

    #[test]
    fn test_ability_specific_effects_applied() {
        let mut app = App::new();
        app.init_resource::<AbilityCatalog>()
            .add_plugins(MinimalPlugins)
            .add_plugins(CommandsPlugin)
            .add_plugins(AbilityCastingPlugin)
            .add_plugins(OngoingCastPlugin)
            .add_plugins(WeaponAttackPlugin)
            .add_plugins(NeedlingHexPlugin)
            .add_plugins(FightPlugin)
            .add_plugins(DamageResolutionPlugin);

        let TestFightEntities {
            fight_e: _,
            caster_e: _,
            slot_e,
            ability_e: attack_ability_e,
            enemy_e,
        } = spawn_test_fight(&mut app);

        // 1. Test Attack
        app.world_mut().trigger(OngoingCastFinishedSuccessfully {
            slot_entity: slot_e,
            ability_entity: attack_ability_e,
            cast_target: Some(enemy_e),
        });

        app.update();

        // Verify DealDamage event
        let events = app.world().resource::<Messages<DealDamage>>();
        assert!(!events.is_empty(), "Should have DealDamage event");

        // Verify NO NeedlingHexEffect
        let has_effects = app.world().get::<HasEffects>(enemy_e);
        assert!(has_effects.is_none(), "Target should have NO HasEffects");

        // Clear events
        app.world_mut()
            .resource_mut::<Messages<DealDamage>>()
            .clear();

        // 2. Test Needling Hex
        let hex_ability_e = app
            .world_mut()
            .spawn((
                Ability {
                    id: AbilityId::NeedlingHex,
                    name: "Needling Hex".into(),
                    description: "Needling Hex".into(),
                },
                NeedlingHexAbility,
            ))
            .id();

        app.world_mut().trigger(OngoingCastFinishedSuccessfully {
            slot_entity: slot_e,
            ability_entity: hex_ability_e,
            cast_target: Some(enemy_e),
        });

        app.update();
        app.update();

        // Verify NeedlingHexEffect
        // NeedlingHexEffect is not on the target, but on a child of the holder.
        // We check if HasEffects is present, and if we can find the effect.

        let has_effects = app.world().get::<HasEffects>(enemy_e);
        assert!(has_effects.is_some(), "Target should have HasEffects");

        let holder_e = has_effects.unwrap().holder();
        let children = app.world().get::<Children>(holder_e);
        assert!(children.is_some(), "Holder should have children");

        let found = children
            .unwrap()
            .iter()
            .any(|child| app.world().get::<NeedlingHexEffect>(child).is_some());
        assert!(
            found,
            "Should find NeedlingHexEffect on a child of the holder"
        );
    }
}
