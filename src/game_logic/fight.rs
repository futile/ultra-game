use bevy::{ecs::system::SystemParam, prelude::*, time::Stopwatch, utils::HashSet};

use super::{
    commands::{GameCommand, GameCommandSource},
    faction::Faction,
    health::{Health, LivenessChangeEvent},
};
use crate::PerUpdateSet;

#[derive(Debug, Default, Clone, Component, Reflect)]
pub struct Fight;

#[derive(Debug, Default, PartialEq, Eq, Component, Reflect)]
pub enum FightEndCondition {
    #[default]
    SingleFactionSurvives,
}

#[derive(Debug, Default, Component, Reflect)]
pub struct FightTime {
    pub stop_watch: Stopwatch,
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

#[derive(Debug, Component, Reflect)]
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
}

impl<'w, 's> FightInterface<'w, 's> {
    pub fn get_fight_status(&self, fight_e: Entity) -> FightStatus {
        let (_, fight_result) = self.fights.get(fight_e).unwrap();

        match fight_result {
            None => FightStatus::Ongoing,
            Some(_) => FightStatus::Ended,
        }
    }
}

fn single_faction_survives_check(
    mut commands: Commands,
    mut liveness_events: EventReader<LivenessChangeEvent>,
    fight_end_conditions: Query<&FightEndCondition, (With<Fight>, Without<FightResult>)>,
    parents: Query<&Parent>,
    childrens: Query<&Children>,
    health_factions: Query<(&Health, &Faction)>,
) {
    let mut fights_to_check: HashSet<Entity> = HashSet::new();

    for liveness_change in liveness_events.read() {
        let died_entity = match liveness_change {
            LivenessChangeEvent::EntityDied { which } => *which,
        };

        let fight_e = parents.get(died_entity).unwrap().get();

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
    trigger: Trigger<GameCommand>,
    mut fight_times: Query<&mut FightTime>,
) {
    if trigger.event().source == GameCommandSource::UserInteraction {
        fight_times
            .get_mut(trigger.entity())
            .unwrap()
            .stop_watch
            .unpause();
    }
}

fn on_add_fight(trigger: Trigger<OnAdd, Fight>, mut commands: Commands) {
    commands
        .entity(trigger.entity())
        .observe(unpause_fight_on_user_command);
}

pub struct FightPlugin;

impl Plugin for FightPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Fight>()
            .register_type::<FightEndCondition>()
            .register_type::<FightResult>()
            .register_type::<FightTime>()
            .add_systems(
                FixedUpdate,
                (
                    tick_fight_times.in_set(PerUpdateSet::TimeUpdate),
                    (single_faction_survives_check, pause_just_ended_fights)
                        .chain()
                        .in_set(PerUpdateSet::FightEndChecking),
                ),
            )
            .observe(on_add_fight);
    }
}
