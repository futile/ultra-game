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

const THIS_ABILITY_ID: AbilityId = AbilityId::Attack;
const THIS_ABILITY_DAMAGE: f64 = 10.0;
const THIS_ABILITY_ABILITY_COOLDOWN: Duration = Duration::from_secs(5);

fn spawn_weapon_attack(commands: &mut Commands) -> Entity {
    commands
        .spawn((
            Ability {
                name: "Attack".into(),
                description: format!(
                    "Strike with your weapon, dealing {THIS_ABILITY_DAMAGE} damage."
                )
                .into(),
            },
            THIS_ABILITY_ID,
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

// I need to query caster.
// fn on_weapon_attack(trigger: On<PerformAbility>, query: Query<&Held<Ability>>, ...)
// But Held<Ability> is on the ability entity?
// No, Held<T> is on the held entity (ability), pointing to holder (caster).
// So `Held<Ability>` component on `ability_entity` contains `held_by` (caster).

fn on_weapon_attack(
    trigger: On<PerformAbility>,
    mut deal_damage_events: MessageWriter<DealDamage>,
    abilities: Query<&Held<Ability>>,
) {
    let event = trigger.event();
    let ability_e = event.ability_entity;

    let Ok(held) = abilities.get(ability_e) else {
        warn!("Weapon Attack ability not held by anyone?");
        return;
    };
    let caster_e = held.held_by;

    let Some(target_e) = event.target else {
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
        app.add_systems(Startup, register_ability)
            .add_observer(on_weapon_attack);
    }
}
