use bevy::{prelude::*, utils::HashSet};

use super::{
    faction::Faction,
    health::{Health, LivenessChangeEvent},
};
use crate::PerUpdateSet;

#[derive(Debug, Clone, Component, Reflect)]
pub struct Fight;

#[derive(Debug, PartialEq, Eq, Component, Reflect)]
pub enum FightEndCondition {
    SingleFactionSurvives,
}

#[derive(Debug, Component, Reflect)]
pub enum FightResult {
    FactionVictory { which: Faction },
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

pub struct FightPlugin;

impl Plugin for FightPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Fight>()
            .register_type::<FightEndCondition>()
            .register_type::<FightResult>()
            .add_systems(
                Update,
                single_faction_survives_check.in_set(PerUpdateSet::FightEndChecking),
            );
    }
}
