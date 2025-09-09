use bevy::prelude::*;
use big_brain::prelude::*;

use super::{
    ability::AbilityId,
    ability_casting::{AbilityCastingInterface, UseAbility},
    ability_slots::AbilitySlot,
    commands::{GameCommand, GameCommandKind, GameCommandSource},
};
use crate::utils::holds_held::Holds;

#[derive(Component, Debug, Clone, ScorerBuilder)]
pub struct CanAttackPlayerScorer;

pub fn can_attack_player_scorer_system(
    mut scorers: Query<(&Actor, &mut Score), With<CanAttackPlayerScorer>>,
    ability_casting_interface: AbilityCastingInterface,
    ability_holders: Query<&Holds<AbilityId>>,
    slot_holders: Query<&Holds<AbilitySlot>>,
    ability_ids: Query<&AbilityId>,
    ability_slots: Query<&AbilitySlot>,
) {
    let fight_interface = &ability_casting_interface.fight_interface;

    for (Actor(actor), mut score) in scorers.iter_mut() {
        // Find the fight entity
        let fight_e = fight_interface.get_fight_of_entity(*actor);

        // Don't score if fight is paused
        if fight_interface.is_fight_paused(fight_e) {
            score.set(0.0);
            continue;
        }

        // Find Attack ability entity using iter_descendants
        let attack_ability_entity = ability_holders.iter_descendants(*actor).find(|&ability_e| {
            ability_ids
                .get(ability_e)
                .is_ok_and(|id| *id == AbilityId::Attack)
        });

        let Some(ability_e) = attack_ability_entity else {
            score.set(0.0);
            continue;
        };

        // Find WeaponAttack slot entity using iter_descendants
        let weapon_slot_entity = slot_holders.iter_descendants(*actor).find(|&slot_e| {
            ability_slots
                .get(slot_e)
                .is_ok_and(|slot| slot.tpe == super::ability_slots::AbilitySlotType::WeaponAttack)
        });

        let Some(slot_e) = weapon_slot_entity else {
            score.set(0.0);
            continue;
        };

        // Create UseAbility request to validate
        let use_ability = UseAbility {
            caster_e: *actor,
            slot_e,
            ability_e,
            fight_e,
        };

        // Check if we can cast the ability
        if ability_casting_interface
            .is_valid_cast(&use_ability)
            .is_ok()
        {
            score.set(1.0);
        } else {
            score.set(0.0);
        }
    }
}

#[derive(Component, Debug, Clone, ActionBuilder)]
pub struct AttackPlayerAction;

pub fn attack_player_action_system(
    mut actions: Query<(&Actor, &mut ActionState), With<AttackPlayerAction>>,
    mut game_commands: EventWriter<GameCommand>,
    ability_casting_interface: AbilityCastingInterface,
    ability_holders: Query<&Holds<AbilityId>>,
    slot_holders: Query<&Holds<AbilitySlot>>,
    ability_ids: Query<&AbilityId>,
    ability_slots: Query<&AbilitySlot>,
) {
    let fight_interface = &ability_casting_interface.fight_interface;

    for (Actor(actor), mut action_state) in actions.iter_mut() {
        match *action_state {
            ActionState::Requested => {
                let fight_e = fight_interface.get_fight_of_entity(*actor);

                // Don't act if fight is paused
                if fight_interface.is_fight_paused(fight_e) {
                    *action_state = ActionState::Failure;
                    continue;
                }

                // Find the enemy's Attack ability and WeaponAttack slot
                let attack_ability_entity =
                    ability_holders.iter_descendants(*actor).find(|&ability_e| {
                        ability_ids
                            .get(ability_e)
                            .is_ok_and(|id| *id == AbilityId::Attack)
                    });

                let Some(ability_e) = attack_ability_entity else {
                    *action_state = ActionState::Failure;
                    continue;
                };

                let weapon_slot_entity = slot_holders.iter_descendants(*actor).find(|&slot_e| {
                    ability_slots.get(slot_e).is_ok_and(|slot| {
                        slot.tpe == super::ability_slots::AbilitySlotType::WeaponAttack
                    })
                });

                let Some(slot_e) = weapon_slot_entity else {
                    *action_state = ActionState::Failure;
                    continue;
                };

                // Create and send the game command
                let use_ability = UseAbility {
                    caster_e: *actor,
                    slot_e,
                    ability_e,
                    fight_e,
                };

                if let Err(e) = ability_casting_interface.is_valid_cast(&use_ability) {
                    warn!("invalid `CastAbility`: {use_ability:#?}, reason: {e}");
                    *action_state = ActionState::Failure;
                    continue;
                }

                let game_command = GameCommand::new(
                    GameCommandSource::AIAction,
                    GameCommandKind::UseAbility(use_ability),
                );

                game_commands.write(game_command);
                *action_state = ActionState::Success;
            }
            ActionState::Executing => {
                // Action completes immediately since we just send the command
                error!("Executing reached, should not happen");
                *action_state = ActionState::Failure;
            }
            ActionState::Cancelled => {
                *action_state = ActionState::Failure;
            }
            _ => {}
        }
    }
}

pub struct AiBehaviorPlugin;

impl Plugin for AiBehaviorPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PreUpdate,
            can_attack_player_scorer_system.in_set(BigBrainSet::Scorers),
        )
        .add_systems(
            PreUpdate,
            attack_player_action_system.in_set(BigBrainSet::Actions),
        );
    }
}
