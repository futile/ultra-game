use bevy::prelude::*;

#[derive(Debug, Clone, Component, Reflect, PartialEq, Eq, Hash, derive_more::Display)]
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
        match Iterator::exactly_one(
            factions
                .iter()
                .filter(|(_, other_faction)| self.is_enemy(other_faction)),
        ) {
            Some((e, f)) => (e, f.clone()),
            None => {
                panic!("expected exactly one enemy, but failed.",);
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
