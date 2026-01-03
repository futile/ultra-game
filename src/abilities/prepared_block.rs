// TODO: probably use needling_hex.rs as a basis for this spell
//
// Intended function: Cast time ~1s, -> Channel 3-5s, block 1 attack, up to X dmg

use std::time::Duration;

use bevy::prelude::*;

use super::AbilityCatalog;
use crate::{
    PerUpdateSet,
    game_logic::{
        ability::{
            Ability, AbilityCastTime, AbilityCooldown, AbilityId, AbilitySlotRequirement,
            PerformAbility,
        },
        ability_slots::AbilitySlotType,
        damage_resolution::{DamageInstance, DealDamage},
        effects::{GameEffect, ReflectGameEffect, UniqueEffectInterface},
        fight::FightInterface,
    },
    utils::FiniteRepeatingTimer,
};

// Marker component for Prepared Block ability
#[derive(Component, Debug, Reflect)]
pub struct PreparedBlockAbility;

const THIS_ABILITY_ID: AbilityId = AbilityId::PreparedBlock;
const THIS_ABILITY_ABILITY_COOLDOWN: Duration = Duration::from_secs(30);

fn spawn_prepared_block(commands: &mut Commands) -> Entity {
    commands
        .spawn((
            Ability {
                id: THIS_ABILITY_ID,
                name: "Prepared Block".into(),
                description: "Prepare to block the next hit you would take (up to a certain amount of damage).".into(),
            },
            PreparedBlockAbility,
            AbilitySlotRequirement(AbilitySlotType::ShieldDefend),
            AbilityCooldown {
                duration: THIS_ABILITY_ABILITY_COOLDOWN,
            },
            AbilityCastTime(Duration::from_secs(1)),
        ))
        .id()
}

fn register_ability(catalog: Res<AbilityCatalog>) {
    catalog.register(THIS_ABILITY_ID, spawn_prepared_block);
}

#[derive(Debug, Component, Reflect, Deref, DerefMut)]
#[reflect(GameEffect)]
pub struct PreparedBlockEffect(FiniteRepeatingTimer);

impl GameEffect for PreparedBlockEffect {}

impl PreparedBlockEffect {
    pub const TICK_INTERVAL: Duration = Duration::from_millis(500);
    pub const NUM_TICKS: u32 = 5;

    pub const DMG_PER_TICK: f64 = 5.0;

    fn new() -> PreparedBlockEffect {
        PreparedBlockEffect(FiniteRepeatingTimer::new(
            Self::TICK_INTERVAL,
            Self::NUM_TICKS,
        ))
    }
}

fn on_prepared_block(
    trigger: On<PerformAbility>,
    mut effects_interface: UniqueEffectInterface<PreparedBlockEffect>,
    abilities: Query<(), With<PreparedBlockAbility>>,
) {
    let event = trigger.event();

    let Ok(_ability_e) = abilities.get(event.ability_entity) else {
        return;
    };

    // Prepared Block requires a caster.
    let Some(caster_e) = event.caster else {
        error!("PreparedBlock without caster - ignoring. Event: {event:?}");
        return;
    };

    // Apply effect
    effects_interface.spawn_or_replace_unique_effect(caster_e, PreparedBlockEffect::new());
}

fn tick_prepared_block_effects(
    mut effects: Query<(Entity, &mut PreparedBlockEffect)>,
    mut effects_interface: UniqueEffectInterface<PreparedBlockEffect>,
    mut deal_damage_events: MessageWriter<DealDamage>,
    fight_interface: FightInterface,
    time: Res<Time>,
) {
    for (effect_e, mut effect) in &mut effects {
        let effect_target = effects_interface.get_target_of_effect(effect_e);
        let fight_e = fight_interface.get_fight_of_entity(effect_target);

        if fight_interface.is_fight_paused(fight_e) {
            continue;
        }

        let just_elapsed_ticks = effect.tick_get_fresh_ticks(time.delta());

        if effect.is_finished() {
            effects_interface.remove_unique_effect(effect_target);
        }

        for _ in 0..just_elapsed_ticks {
            deal_damage_events.write(DealDamage(DamageInstance {
                source: None,
                target: effect_target,
                amount: PreparedBlockEffect::DMG_PER_TICK,
            }));
        }
    }
}

#[derive(Debug)]
pub struct PreparedBlockPlugin;

impl Plugin for PreparedBlockPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<PreparedBlockEffect>()
            .register_type::<PreparedBlockAbility>()
            .add_systems(PreStartup, register_ability)
            .add_systems(
                FixedUpdate,
                tick_prepared_block_effects.in_set(PerUpdateSet::LogicUpdate),
            )
            .add_observer(on_prepared_block);
    }
}
