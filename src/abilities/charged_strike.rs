use std::time::Duration;

use bevy::prelude::*;

use super::AbilityCatalog;
use crate::{
    PerUpdateSet,
    game_logic::{
        ability::{Ability, AbilityId},
        ability_casting::{AbilityCastingInterface, UseAbility},
        ability_slots::{AbilitySlot, AbilitySlotType},
        commands::{GameCommand, GameCommandKind},
        cooldown::Cooldown,
        damage_resolution::{DamageInstance, DealDamage},
        faction::Faction,
        ongoing_cast::{OngoingCast, OngoingCastAborted, OngoingCastFinishedSuccessfully},
    },
};

const THIS_ABILITY_ID: AbilityId = AbilityId::ChargedStrike;
const THIS_ABILITY_ABILITY_COOLDOWN: Duration = Duration::from_secs(20);
const CAST_TIME: Duration = Duration::from_secs(2);

fn add_to_ability_catalog(mut abilties_catalog: ResMut<AbilityCatalog>) {
    abilties_catalog.0.insert(
        THIS_ABILITY_ID,
        Ability {
            name: "Charged Strike".into(),
            id: THIS_ABILITY_ID,
            slot_type: Some(AbilitySlotType::WeaponAttack),
            #[expect(clippy::useless_format, reason = "Uniformity")]
            description: format!("Charge an extra strong strike, dealing 25 damage!").into(),
        },
    );
}

fn cast_ability(
    mut game_commands: EventReader<GameCommand>,
    ability_slots: Query<&AbilitySlot>,
    factions: Query<(Entity, &Faction)>,
    mut ability_casting_interface: AbilityCastingInterface,
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

        if !ability_casting_interface.is_matching_cast(cast, &THIS_ABILITY_ID) {
            continue;
        }

        if let Err(e) = ability_casting_interface.is_valid_cast(cast) {
            warn!("invalid `CastAbility`: {cast:#?}, reason: {e}");
            continue;
        }

        let slot = ability_slots.get(*slot_e).unwrap();
        let (_, faction) = factions.get(*caster_e).unwrap();

        let (target_e, _target_faction) = faction.find_single_enemy(&factions);

        println!(
            "Casting ability: {THIS_ABILITY_ID:?} | Fight: {fight_e:?} | Caster: {caster_e:?} | Slot: {slot_e:?} [{slot:?}] | Target: {target_e:?}"
        );

        let ongoing_cast_e = ability_casting_interface.start_cast(
            *slot_e,
            OngoingCast {
                ability_e: *ability_e,
                cast_timer: Timer::new(CAST_TIME, TimerMode::Once),
            },
        );

        // start cooldown on the ability
        commands
            .entity(*ability_e)
            .insert(Cooldown::new(THIS_ABILITY_ABILITY_COOLDOWN));

        let caster_e = *caster_e;
        commands
            .entity(ongoing_cast_e)
            .observe(
                move |_trigger: Trigger<OngoingCastFinishedSuccessfully>,
                      mut deal_damage_events: EventWriter<DealDamage>| {
                    deal_damage_events.write(DealDamage(DamageInstance {
                        source: Some(caster_e),
                        target: target_e,
                        amount: 25.0,
                    }));
                },
            )
            .observe(
                // for debugging atm.
                move |_trigger: Trigger<OngoingCastAborted>, mut _commands: Commands| {
                    println!("Charged Strike aborted!");

                    // doesn't work, triggers panic
                    // commands.entity(trigger.target()).log_components();
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
