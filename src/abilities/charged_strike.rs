use std::time::Duration;

use bevy::prelude::*;

use super::AbilityCatalog;
use crate::{
    game_logic::{
        ability::{Ability, AbilityId, AbilitySlot, AbilitySlotType},
        commands::{CastAbilityInterface, GameCommand, GameCommandKind, UseAbility},
        damage_resolution::{DamageInstance, DealDamage},
        faction::Faction,
        ongoing_cast::{
            OngoingCast, OngoingCastAborted, OngoingCastFinishedSuccessfully, OngoingCastInterface,
        },
    },
    PerUpdateSet,
};

const THIS_ABILITY_ID: AbilityId = AbilityId::ChargedStrike;

const CAST_TIME: Duration = Duration::from_secs(2);

fn add_to_ability_catalog(mut abilties_catalog: ResMut<AbilityCatalog>) {
    abilties_catalog.0.insert(
        THIS_ABILITY_ID,
        Ability {
            name: "Charged Strike".into(),
            id: THIS_ABILITY_ID,
            slot_type: Some(AbilitySlotType::WeaponAttack),
            #[expect(clippy::useless_format, reason = "Uniformity")]
            description: format!("Charge an extra strong strike!").into(),
        },
    );
}

fn cast_ability(
    mut game_commands: EventReader<GameCommand>,
    ability_slots: Query<&AbilitySlot>,
    factions: Query<(Entity, &Faction)>,
    cast_ability_interface: CastAbilityInterface,
    mut ongoing_cast_interface: OngoingCastInterface,
    mut commands: Commands,
) {
    for cmd in game_commands.read() {
        #[expect(irrefutable_let_patterns, reason = "only one enum variant for now")]
        let GameCommand {
            source: _,
            kind:
                GameCommandKind::UseAbility(
                    cast @ UseAbility {
                        caster_e,
                        slot_e,
                        ability_e,
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

        let ongoing_cast_e = ongoing_cast_interface.start_new_cast(OngoingCast {
            slot_e: slot_e.unwrap(),
            fight_e: *fight_e,
            ability_e: *ability_e,
            cast_timer: Timer::new(CAST_TIME, TimerMode::Once),
        });

        let caster_e = *caster_e;
        commands
            .entity(ongoing_cast_e)
            .observe(
                move |_trigger: Trigger<OngoingCastFinishedSuccessfully>,
                      mut deal_damage_events: EventWriter<DealDamage>| {
                    deal_damage_events.write(DealDamage(DamageInstance {
                        source: Some(caster_e),
                        target: target_e,
                        amount: 51.0,
                    }));
                },
            )
            .observe(
                // for debugging atm.
                move |trigger: Trigger<OngoingCastAborted>, mut commands: Commands| {
                    println!("Charged Strike aborted!");
                    commands.entity(trigger.target()).log_components();
                },
            );

        // fire an event for the executed `GameCommand`
        commands.trigger_targets(cmd.clone(), *fight_e);
    }
}

#[derive(Debug)]
pub struct ChargedStrikePlugin;

impl Plugin for ChargedStrikePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, add_to_ability_catalog)
            .add_systems(Update, cast_ability.in_set(PerUpdateSet::CommandResolution));
    }
}
