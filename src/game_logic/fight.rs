use std::time::Duration;

use bevy::{ecs::system::SystemParam, platform::collections::HashSet, prelude::*, time::Stopwatch};

use crate::{
    PerUpdateSet,
    game_logic::{
        commands::{GameCommand, GameCommandSource},
        faction::Faction,
        health::{Health, LivenessChangeEvent},
    },
};

#[derive(Debug, Default, Clone, Component, Reflect)]
pub struct Fight;

#[derive(Debug, Default, PartialEq, Eq, Component, Reflect)]
pub enum FightEndCondition {
    #[default]
    SingleFactionSurvives,
}

// Starts paused.
#[derive(Debug, Component, Reflect)]
pub struct FightTime {
    stop_watch: Stopwatch,
}

impl FightTime {
    pub fn new() -> Self {
        let mut stop_watch = Stopwatch::new();
        stop_watch.pause();
        Self { stop_watch }
    }

    pub fn is_paused(&self) -> bool {
        self.stop_watch.is_paused()
    }

    pub fn set_paused(&mut self, should_pause: bool) {
        if should_pause {
            self.stop_watch.pause();
        } else {
            self.stop_watch.unpause();
        }
    }

    pub fn stop_watch(&self) -> &Stopwatch {
        &self.stop_watch
    }
}

impl Default for FightTime {
    fn default() -> Self {
        Self::new()
    }
}

fn tick_fight_times(mut fight_times: Query<&mut FightTime>, time: Res<Time>) {
    for mut fight_time in &mut fight_times {
        fight_time.stop_watch.tick(time.delta());
    }
}

#[derive(Debug, Bundle, Default)]
pub struct FightBundle {
    _fight: Fight,
    fight_time: FightTime,
    fight_end_condition: FightEndCondition,
}

impl FightBundle {
    pub fn new() -> FightBundle {
        default()
    }
}

#[derive(Debug, Clone, Component, Reflect)]
pub enum FightResult {
    FactionVictory { which: Faction },
}

#[derive(Debug, PartialEq, Eq, Reflect)]
pub enum FightStatus {
    Ongoing,
    Ended,
}

impl FightStatus {
    pub fn is_ended(&self) -> bool {
        *self == FightStatus::Ended
    }
}

#[derive(SystemParam)]
pub struct FightInterface<'w, 's> {
    fights: Query<'w, 's, (&'static Fight, Option<&'static FightResult>)>,
    fight_times: Query<'w, 's, &'static mut FightTime>,
    parents: Query<'w, 's, &'static ChildOf>,
}

impl<'w, 's> FightInterface<'w, 's> {
    pub fn get_fight_result(&self, fight_e: Entity) -> Option<FightResult> {
        let (_, fight_result) = self.fights.get(fight_e).unwrap();

        fight_result.cloned()
    }

    pub fn get_fight_status(&self, fight_e: Entity) -> FightStatus {
        match self.get_fight_result(fight_e) {
            None => FightStatus::Ongoing,
            Some(_) => FightStatus::Ended,
        }
    }

    /// `entity` must be a direct child of `Fight`
    pub fn get_fight_of_entity(&self, entity: Entity) -> Entity {
        self.parents.get(entity).unwrap().parent()
    }

    pub fn is_fight_paused(&self, fight_e: Entity) -> bool {
        self.fight_times.get(fight_e).unwrap().is_paused()
    }

    pub fn set_fight_paused(&mut self, fight_e: Entity, should_pause: bool) {
        let is_fight_ended = self.get_fight_status(fight_e).is_ended();
        let mut fight_time = self.fight_times.get_mut(fight_e).unwrap();

        if should_pause {
            fight_time.stop_watch.pause()
        } else if !is_fight_ended {
            fight_time.stop_watch.unpause()
        }
    }

    pub fn get_elapsed_fight_time(&self, fight_e: Entity) -> Duration {
        self.fight_times.get(fight_e).unwrap().stop_watch.elapsed()
    }
}

fn single_faction_survives_check(
    mut commands: Commands,
    mut liveness_events: MessageReader<LivenessChangeEvent>,
    fight_end_conditions: Query<&FightEndCondition, (With<Fight>, Without<FightResult>)>,
    parents: Query<&ChildOf>,
    childrens: Query<&Children>,
    health_factions: Query<(&Health, &Faction)>,
) {
    let mut fights_to_check: HashSet<Entity> = HashSet::new();

    for liveness_change in liveness_events.read() {
        let died_entity = match liveness_change {
            LivenessChangeEvent::EntityDied { which } => *which,
        };

        let fight_e = parents.get(died_entity).unwrap().parent();

        if fights_to_check.contains(&fight_e) {
            continue;
        }

        let should_check =
            fight_end_conditions.get(fight_e).unwrap() == &FightEndCondition::SingleFactionSurvives;

        if !should_check {
            continue;
        }

        fights_to_check.insert(fight_e);
    }

    for fight_e in fights_to_check.into_iter() {
        let mut alive_factions: HashSet<Faction> = HashSet::new();

        let fight_children = childrens.get(fight_e).unwrap();

        for (health, faction) in health_factions.iter_many(fight_children) {
            if health.is_alive() {
                alive_factions.insert(faction.clone());
            }
        }

        // TODO: can all factions lose at the same time? Draw?
        assert!(!alive_factions.is_empty());

        if alive_factions.len() == 1 {
            let winning_faction = alive_factions.into_iter().next().unwrap();

            commands
                .entity(fight_e)
                .insert(FightResult::FactionVictory {
                    which: winning_faction,
                });
        }
    }
}

// TODO: Use observer + trigger (with `FightEnded` or similar) instead?
fn pause_just_ended_fights(
    mut just_ended_fight_times: Query<&mut FightTime, (With<Fight>, Added<FightResult>)>,
) {
    for mut fight_time in &mut just_ended_fight_times {
        fight_time.stop_watch.pause();
    }
}

fn unpause_fight_on_user_command(
    mut game_commands: MessageReader<GameCommand>,
    mut fight_times: Query<&mut FightTime>,
) {
    for game_command in game_commands.read() {
        if game_command.source == GameCommandSource::UserInteraction
            && let Some(fight_e) = game_command.kind.get_fight_e() {
                fight_times.get_mut(fight_e).unwrap().stop_watch.unpause();
            }
    }
}

pub struct FightPlugin;

impl Plugin for FightPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Fight>()
            .register_type::<FightEndCondition>()
            .register_type::<FightResult>()
            .register_type::<FightTime>()
            .add_message::<LivenessChangeEvent>()
            .add_systems(
                FixedUpdate,
                (
                    tick_fight_times.in_set(PerUpdateSet::TimeUpdate),
                    (single_faction_survives_check, pause_just_ended_fights)
                        .chain()
                        .in_set(PerUpdateSet::FightEndChecking),
                ),
            )
            .add_systems(
                Update,
                unpause_fight_on_user_command.after(PerUpdateSet::CommandSubmission),
            );
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::{FightPlugin, FightTime};
    use crate::{
        abilities::{AbilityCatalog, weapon_attack::WeaponAttackPlugin},
        game_logic::{
            ability_casting::{AbilityCastingPlugin, UseAbility},
            commands::{CommandsPlugin, GameCommand, GameCommandKind},
            ongoing_cast::OngoingCastPlugin,
        },
        test_utils::{TestFightEntities, spawn_test_fight},
    };

    #[test]
    fn test_fight_timer_unpauses_on_user_command() {
        let mut app = App::new();
        app.init_resource::<AbilityCatalog>()
            .add_plugins(MinimalPlugins)
            .add_plugins(FightPlugin)
            .add_plugins(CommandsPlugin)
            .add_plugins(AbilityCastingPlugin)
            .add_plugins(OngoingCastPlugin)
            .add_plugins(WeaponAttackPlugin);

        let TestFightEntities {
            fight_e,
            caster_e,
            slot_e,
            ability_e,
            enemy_e: _,
        } = spawn_test_fight(&mut app);

        // Verify timer paused
        let fight_time = app.world().get::<FightTime>(fight_e).unwrap();
        assert!(fight_time.is_paused());

        app.world_mut()
            .write_message(GameCommand::new_from_user(GameCommandKind::UseAbility(
                UseAbility {
                    caster_e,
                    slot_e,
                    ability_e,
                    target: None,
                    fight_e,
                },
            )));

        app.update();

        // Verify timer unpaused
        let fight_time = app.world().get::<FightTime>(fight_e).unwrap();
        assert!(!fight_time.is_paused(), "Fight timer should be unpaused");
    }
}
