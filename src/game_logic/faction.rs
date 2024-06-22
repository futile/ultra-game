use bevy::prelude::*;
use itertools::Itertools;

#[derive(Debug, Clone, Component, Reflect, PartialEq, Eq, Hash)]
pub enum Faction {
    Player,
    Enemy,
}

impl Faction {
    pub fn is_friendly(&self, other: &Faction) -> bool {
        self == other
    }

    pub fn is_enemy(&self, other: &Faction) -> bool {
        !self.is_friendly(other)
    }

    pub fn find_single_enemy(&self, factions: &Query<(Entity, &Faction)>) -> (Entity, Faction) {
        match factions
            .iter()
            .filter(|(_, other_faction)| self.is_enemy(other_faction))
            .exactly_one()
        {
            Ok((e, f)) => return (e, f.clone()),
            Err(multiple) => {
                panic!(
                    "expected exactly one enemy, but found multiple: {:?}",
                    multiple.try_len()
                );
            }
        }
    }
}

pub struct FactionPlugin;

impl Plugin for FactionPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Faction>();
    }
}
