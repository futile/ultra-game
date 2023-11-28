use bevy::prelude::*;

#[derive(Debug, Clone, Component)]
pub struct Fight {
    pub player_character: Entity,
    pub enemy: Entity,
}

#[derive(Debug, Component)]
pub struct PlayerCharacter;

#[derive(Debug, Component)]
pub struct Enemy;
