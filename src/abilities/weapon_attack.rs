use std::time::Duration;

use bevy::prelude::*;

use super::AbilityCatalog;
use crate::{
    game_logic::{
        ability::{
            Ability, AbilityCastTime, AbilityCooldown, AbilityId, AbilitySlotRequirement,
            PerformAbility,
        },
        ability_slots::AbilitySlotType,
        damage_resolution::{DamageInstance, DealDamage},
    },
    utils::holds_held::Held,
};

// Marker component for weapon attack ability
#[derive(Component, Debug, Reflect)]
pub struct WeaponAttackAbility;

const THIS_ABILITY_ID: AbilityId = AbilityId::WeaponAttack;
const THIS_ABILITY_DAMAGE: f64 = 10.0;
const THIS_ABILITY_ABILITY_COOLDOWN: Duration = Duration::from_secs(5);

pub fn spawn_weapon_attack(commands: &mut Commands) -> Entity {
    commands
        .spawn((
            Ability {
                id: THIS_ABILITY_ID,
                name: "Attack".into(),
                description: format!(
                    "Strike with your weapon, dealing {THIS_ABILITY_DAMAGE} damage."
                )
                .into(),
            },
            WeaponAttackAbility,
            AbilitySlotRequirement(AbilitySlotType::WeaponAttack),
            AbilityCooldown {
                duration: THIS_ABILITY_ABILITY_COOLDOWN,
            },
            AbilityCastTime(Duration::ZERO),
        ))
        .id()
}

fn register_ability(catalog: Res<AbilityCatalog>) {
    catalog.register(THIS_ABILITY_ID, spawn_weapon_attack);
}

fn on_weapon_attack(
    trigger: On<PerformAbility>,
    mut deal_damage_events: MessageWriter<DealDamage>,
    abilities: Query<&Held<Ability>, With<WeaponAttackAbility>>,
) {
    let event = trigger.event();

    let Ok(_ability_e) = abilities.get(event.ability_entity) else {
        return;
    };

    let Some(caster_e) = abilities.related::<Held<Ability>>(event.ability_entity) else {
        error!("Weapon Attack ability holder not found? Event: {event:?}");
        return;
    };

    let Some(target_e) = event.target else {
        error!("Weapon Attack ability performed without a target!");
        return;
    };

    // Deal damage
    deal_damage_events.write(DealDamage(DamageInstance {
        source: Some(caster_e),
        target: target_e,
        amount: THIS_ABILITY_DAMAGE,
    }));
}

#[derive(Debug)]
pub struct WeaponAttackPlugin;

impl Plugin for WeaponAttackPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<WeaponAttackAbility>()
            .add_systems(PreStartup, register_ability)
            .add_observer(on_weapon_attack);
    }
}
