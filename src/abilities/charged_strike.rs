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

const THIS_ABILITY_ID: AbilityId = AbilityId::ChargedStrike;
const THIS_ABILITY_ABILITY_COOLDOWN: Duration = Duration::from_secs(20);
const CAST_TIME: Duration = Duration::from_secs(2);

fn spawn_charged_strike(commands: &mut Commands) -> Entity {
    commands
        .spawn((
            Ability {
                name: "Charged Strike".into(),
                description: format!("Charge an extra strong strike, dealing 25 damage!").into(),
            },
            THIS_ABILITY_ID,
            AbilitySlotRequirement(AbilitySlotType::WeaponAttack),
            AbilityCooldown {
                duration: THIS_ABILITY_ABILITY_COOLDOWN,
            },
            AbilityCastTime(CAST_TIME),
        ))
        .id()
}

fn register_ability(catalog: Res<AbilityCatalog>) {
    catalog.register(THIS_ABILITY_ID, spawn_charged_strike);
}

fn on_charged_strike(
    trigger: On<PerformAbility>,
    mut deal_damage_events: MessageWriter<DealDamage>,
    abilities: Query<&Held<Ability>>,
) {
    let event = trigger.event();
    let ability_e = event.ability_entity;

    let Ok(held) = abilities.get(ability_e) else {
        warn!("Charged Strike ability not held by anyone?");
        return;
    };
    let caster_e = held.held_by;

    let Some(target_e) = event.target else {
        return;
    };

    deal_damage_events.write(DealDamage(DamageInstance {
        source: Some(caster_e),
        target: target_e,
        amount: 25.0,
    }));
}

#[derive(Debug)]
pub struct ChargedStrikePlugin;

impl Plugin for ChargedStrikePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, register_ability)
            .add_observer(on_charged_strike);
    }
}
