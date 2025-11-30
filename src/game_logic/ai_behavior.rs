use bevy::prelude::*;
use big_brain::prelude::*;

use super::{
    ability::{Ability, AbilityId},
    ability_casting::{AbilityCastingInterface, UseAbility},
    ability_slots::{AbilitySlot, AbilitySlotType},
    commands::{GameCommand, GameCommandKind, GameCommandSource},
    faction::Faction,
    fight::Fight,
};
use crate::utils::holds_held::Holds;

#[derive(Component, Debug, Clone, ScorerBuilder)]
pub struct CanAttackPlayerScorer;

#[allow(
    clippy::too_many_arguments,
    reason = "it's a system, many arguments is ok"
)]
pub fn can_attack_player_scorer_system(
    mut scorers: Query<(&Actor, &mut Score), With<CanAttackPlayerScorer>>,
    ability_casting_interface: AbilityCastingInterface,
    ability_holders: Query<&Holds<Ability>>,
    slot_holders: Query<&Holds<AbilitySlot>>,
    abilities: Query<&Ability>,
    ability_slots: Query<&AbilitySlot>,
    fights: Query<&Children, With<Fight>>,
    factions: Query<&Faction>,
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
            abilities
                .get(ability_e)
                .is_ok_and(|ability| ability.id == AbilityId::WeaponAttack)
        });

        let Some(ability_e) = attack_ability_entity else {
            score.set(0.0);
            continue;
        };

        // Find WeaponAttack slot entity using iter_descendants
        let weapon_slot_entity = slot_holders.iter_descendants(*actor).find(|&slot_e| {
            ability_slots
                .get(slot_e)
                .is_ok_and(|slot| slot.tpe == AbilitySlotType::WeaponAttack)
        });

        let Some(slot_e) = weapon_slot_entity else {
            score.set(0.0);
            continue;
        };

        let Ok(fight_children) = fights.get(fight_e) else {
            score.set(0.0);
            continue;
        };

        let own_faction = factions.get(*actor).ok();
        let target_e = fight_children.iter().find(|child| {
            factions
                .get(*child)
                .ok()
                .is_some_and(|f| own_faction.is_some_and(|own| f != own))
        });

        let Some(target_e) = target_e else {
            score.set(0.0);
            continue;
        };
        // Create UseAbility request to validate
        let use_ability = UseAbility {
            caster_e: *actor,
            slot_e,
            ability_e,
            target: Some(target_e),
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

#[allow(
    clippy::too_many_arguments,
    reason = "it's a system, many arguments is ok"
)]
pub fn attack_player_action_system(
    mut actions: Query<(&Actor, &mut ActionState), With<AttackPlayerAction>>,
    mut game_commands: MessageWriter<GameCommand>,
    ability_casting_interface: AbilityCastingInterface,
    ability_holders: Query<&Holds<Ability>>,
    slot_holders: Query<&Holds<AbilitySlot>>,
    abilities: Query<&Ability>,
    ability_slots: Query<&AbilitySlot>,
    fights: Query<&Children, With<Fight>>,
    factions: Query<&Faction>,
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
                        abilities
                            .get(ability_e)
                            .is_ok_and(|ability| ability.id == AbilityId::WeaponAttack)
                    });

                let Some(ability_e) = attack_ability_entity else {
                    *action_state = ActionState::Failure;
                    continue;
                };

                let weapon_slot_entity = slot_holders.iter_descendants(*actor).find(|&slot_e| {
                    ability_slots
                        .get(slot_e)
                        .is_ok_and(|slot| slot.tpe == AbilitySlotType::WeaponAttack)
                });

                let Some(slot_e) = weapon_slot_entity else {
                    *action_state = ActionState::Failure;
                    continue;
                };

                let Ok(fight_children) = fights.get(fight_e) else {
                    *action_state = ActionState::Failure;
                    continue;
                };

                let own_faction = factions.get(*actor).ok();
                let target_e = fight_children.iter().find(|child| {
                    factions
                        .get(*child)
                        .ok()
                        .is_some_and(|f| own_faction.is_some_and(|own| f != own))
                });

                let Some(target_e) = target_e else {
                    *action_state = ActionState::Failure;
                    continue;
                };
                // Create and send the game command
                let use_ability = UseAbility {
                    caster_e: *actor,
                    slot_e,
                    ability_e,
                    target: Some(target_e),
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

                // Execution finishes immediately
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

#[cfg(test)]
mod tests {
    use bevy::{log::LogPlugin, prelude::*};
    use big_brain::{BigBrainPlugin, prelude::*};

    use super::*;
    use crate::{
        game_logic::{
            ability_casting::AbilityCastingPlugin,
            commands::{CommandsPlugin, GameCommand, GameCommandKind},
            fight::{FightPlugin, FightTime},
        },
        test_utils::{TestFightEntities, spawn_test_fight},
    };

    #[test]
    fn test_ai_attacks_immediately() {
        let mut app = App::new();

        app.add_plugins(MinimalPlugins)
            .add_plugins(LogPlugin::default())
            .add_plugins(BigBrainPlugin::new(PreUpdate))
            .add_plugins(AiBehaviorPlugin)
            .add_plugins(FightPlugin)
            .add_plugins(AbilityCastingPlugin)
            .add_plugins(CommandsPlugin);

        let TestFightEntities {
            fight_e,
            caster_e, // This will be target
            slot_e: _,
            ability_e: _,
            enemy_e, // This will be our AI (Player)
        } = spawn_test_fight(&mut app);

        // Configure AI (enemy_e)
        app.world_mut().entity_mut(enemy_e).insert((Thinker::build()
            .picker(FirstToScore { threshold: 0.5 })
            .when(CanAttackPlayerScorer, AttackPlayerAction),));

        // Unpause fight
        app.world_mut()
            .get_mut::<FightTime>(fight_e)
            .unwrap()
            .set_paused(false);

        // currently takes this many updates until the action is executed. adjust as necessary
        // after updates etc.
        for _ in 0..4 {
            app.update();
        }

        // Check for GameCommand
        let mut events = app.world_mut().resource_mut::<Messages<GameCommand>>();
        let commands: Vec<GameCommand> = events.drain().collect();

        assert!(!commands.is_empty(), "AI should have submitted a command");

        let command = &commands[0];
        match &command.kind {
            GameCommandKind::UseAbility(use_ability) => {
                assert_eq!(use_ability.caster_e, enemy_e);
                assert_eq!(use_ability.target, Some(caster_e));
            }
        }
    }
}
