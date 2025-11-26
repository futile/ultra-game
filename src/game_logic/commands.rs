use bevy::prelude::*;
use derive_more::From;

use crate::game_logic::ability_casting::UseAbility;

#[derive(Event, Message, Debug, Clone)]
pub struct GameCommand {
    pub source: GameCommandSource,
    pub kind: GameCommandKind,
}

impl GameCommand {
    pub fn new(source: GameCommandSource, kind: GameCommandKind) -> Self {
        Self { source, kind }
    }

    pub fn new_from_user(kind: GameCommandKind) -> Self {
        Self::new(GameCommandSource::UserInteraction, kind)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GameCommandSource {
    UserInteraction,
    AIAction,
}

#[derive(Debug, Clone, From)]
pub enum GameCommandKind {
    UseAbility(UseAbility),
}

impl GameCommandKind {
    pub fn get_fight_e(&self) -> Option<Entity> {
        match self {
            GameCommandKind::UseAbility(use_ability) => Some(use_ability.fight_e),
        }
    }
}

pub struct CommandsPlugin;

impl Plugin for CommandsPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<GameCommand>();
    }
}
