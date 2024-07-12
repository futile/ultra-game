use std::time::Duration;

use bevy::prelude::*;

use super::AbilityCatalog;
use crate::{
    game_logic::{
        commands::{CastAbility, CastAbilityInterface, GameCommand, GameCommandKind},
        effects::EffectsInterface,
        faction::Faction,
        Ability, AbilityId, AbilitySlot,
    },
    utils::FiniteRepeatingTimer,
    PerUpdateSet,
};

const THIS_ABILITY_ID: AbilityId = AbilityId::NeedlingHex;

const THIS_ABILITY_DAMAGE: f64 = 51.0;

fn add_to_ability_catalog(mut abilties_catalog: ResMut<AbilityCatalog>) {
    abilties_catalog.0.insert(
        THIS_ABILITY_ID,
        Ability {
            name: "Needling Hex".into(),
            id: THIS_ABILITY_ID,
            slot_type: None,
            description: format!("Hex your enemy with repeated damage. {THIS_ABILITY_DAMAGE}")
                .into(),
        },
    );
}

#[derive(Debug, Component, Reflect, Deref, DerefMut)]
struct NeedlingHexEffect(FiniteRepeatingTimer);

impl NeedlingHexEffect {
    const TICK_INTERVAL: Duration = Duration::from_millis(500);
    const NUM_TICKS: u32 = 5;

    #[expect(dead_code, reason = "wip")]
    const DMG_PER_TICK: f64 = 10.0;

    fn new() -> NeedlingHexEffect {
        NeedlingHexEffect(FiniteRepeatingTimer::new(
            Self::TICK_INTERVAL,
            Self::NUM_TICKS,
        ))
    }
}

fn cast_ability(
    mut game_commands: EventReader<GameCommand>,
    ability_slots: Query<&AbilitySlot>,
    factions: Query<(Entity, &Faction)>,
    cast_ability_interface: CastAbilityInterface,
    mut effects_interface: EffectsInterface,
    mut commands: Commands,
) {
    for cmd in game_commands.read() {
        #[expect(irrefutable_let_patterns, reason = "only one enum variant for now")]
        let GameCommand {
            source: _,
            kind:
                GameCommandKind::CastAbility(
                    cast @ CastAbility {
                        caster_e,
                        slot_e,
                        ability_e: _,
                        fight_e,
                    },
                ),
        } = cmd
        else {
            continue;
        };

        if !cast_ability_interface.is_matching_cast(cast, &THIS_ABILITY_ID) {
            continue;
        }

        if !cast_ability_interface.is_valid_cast(cast) {
            warn!("invalid `CastAbility`: {cast:#?}");
            continue;
        }

        let slot: Option<&AbilitySlot> = slot_e.map(|slot_e| ability_slots.get(slot_e).unwrap());
        let (_, faction) = factions.get(*caster_e).unwrap();

        let (target_e, _target_faction) = faction.find_single_enemy(&factions);

        println!(
            "Casting ability: {THIS_ABILITY_ID:?} | Fight: {fight_e:?} | Caster: {caster_e:?} | Slot: {slot_e:?} [{slot:?}] | Target: {target_e:?}"
        );

        let effect_e = effects_interface.spawn_effect(target_e);
        commands.entity(effect_e).insert(NeedlingHexEffect::new());

        // fire an event for the executed `GameCommand`
        commands.trigger_targets(cmd.clone(), *fight_e);
    }
}

#[derive(Debug)]
pub struct NeedlingHexPlugin;

impl Plugin for NeedlingHexPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<NeedlingHexEffect>()
            .add_systems(Startup, add_to_ability_catalog)
            .add_systems(Update, cast_ability.in_set(PerUpdateSet::CommandResolution));
    }
}
