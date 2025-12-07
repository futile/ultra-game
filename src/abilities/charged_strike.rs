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

// Marker component for charged strike ability
#[derive(Component, Debug, Reflect)]
pub struct ChargedStrikeAbility;

const THIS_ABILITY_ID: AbilityId = AbilityId::ChargedStrike;

fn spawn_charged_strike(commands: &mut Commands) -> Entity {
    commands
        .spawn((
            Ability {
                id: THIS_ABILITY_ID,
                name: "Charged Strike".into(),
                description: "Charge an extra strong strike, dealing 25 damage!".into(),
            },
            ChargedStrikeAbility,
            AbilitySlotRequirement(AbilitySlotType::WeaponAttack),
            AbilityCooldown {
                duration: Duration::from_secs(20),
            },
            AbilityCastTime(Duration::from_secs(2)),
        ))
        .id()
}

fn register_ability(catalog: Res<AbilityCatalog>) {
    catalog.register(THIS_ABILITY_ID, spawn_charged_strike);
}

fn on_charged_strike(
    trigger: On<PerformAbility>,
    mut deal_damage_events: MessageWriter<DealDamage>,
    abilities: Query<&Held<Ability>, With<ChargedStrikeAbility>>,
) {
    let event = trigger.event();

    let Ok(_ability_e) = abilities.get(event.ability_entity) else {
        return;
    };

    let Some(caster_e) = abilities.related::<Held<Ability>>(event.ability_entity) else {
        error!("Charged Strike ability holder not found? Event: {event:?}");
        return;
    };

    let Some(target_e) = event.target else {
        error!("Charged Strike performed without a target");
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
        app.register_type::<ChargedStrikeAbility>()
            .add_systems(Startup, register_ability)
            .add_observer(on_charged_strike);
    }
}
