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
        faction::Faction,
        fight::FightInterface,
    },
    utils::{FiniteRepeatingTimer, holds_held::Held},
};

const THIS_ABILITY_ID: AbilityId = AbilityId::NeedlingHex;
const THIS_ABILITY_ABILITY_COOLDOWN: Duration = Duration::from_secs(30);

fn spawn_needling_hex(commands: &mut Commands) -> Entity {
    commands
        .spawn((
            Ability {
                name: "Needling Hex".into(),
                description: format!("Hex your enemy with repeated damage over time.").into(),
            },
            THIS_ABILITY_ID,
            AbilitySlotRequirement(AbilitySlotType::Magic),
            AbilityCooldown {
                duration: THIS_ABILITY_ABILITY_COOLDOWN,
            },
            AbilityCastTime(Duration::ZERO),
        ))
        .id()
}

fn register_ability(catalog: Res<AbilityCatalog>) {
    catalog.register(THIS_ABILITY_ID, spawn_needling_hex);
}

#[derive(Debug, Component, Reflect, Deref, DerefMut)]
#[reflect(GameEffect)]
pub struct NeedlingHexEffect(FiniteRepeatingTimer);

impl GameEffect for NeedlingHexEffect {}

impl NeedlingHexEffect {
    pub const TICK_INTERVAL: Duration = Duration::from_millis(500);
    pub const NUM_TICKS: u32 = 5;

    pub const DMG_PER_TICK: f64 = 5.0;

    fn new() -> NeedlingHexEffect {
        NeedlingHexEffect(FiniteRepeatingTimer::new(
            Self::TICK_INTERVAL,
            Self::NUM_TICKS,
        ))
    }
}

fn on_needling_hex(
    trigger: On<PerformAbility>,
    mut effects_interface: UniqueEffectInterface<NeedlingHexEffect>,
    abilities: Query<&Held<Ability>>,
) {
    let event = trigger.event();
    let ability_e = trigger.target;

    // Needling Hex needs a target.
    let Some(target_e) = event.target else {
        return;
    };

    // Apply effect
    effects_interface.spawn_or_replace_unique_effect(target_e, NeedlingHexEffect::new());
}

fn tick_needling_hex_effects(
    mut effects: Query<(Entity, &mut NeedlingHexEffect)>,
    mut effects_interface: UniqueEffectInterface<NeedlingHexEffect>,
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
                amount: NeedlingHexEffect::DMG_PER_TICK,
            }));
        }
    }
}

#[derive(Debug)]
pub struct NeedlingHexPlugin;

impl Plugin for NeedlingHexPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<NeedlingHexEffect>()
            .add_systems(Startup, register_ability)
            .add_systems(
                FixedUpdate,
                tick_needling_hex_effects.in_set(PerUpdateSet::LogicUpdate),
            )
            .add_observer(on_needling_hex);
    }
}
