use bevy::prelude::*;

#[derive(Debug, Event)]
pub struct CastAbility {
    pub caster: Entity,
    pub ability: Entity,
}

pub struct CommandsPlugin;

impl Plugin for CommandsPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<CastAbility>();
    }
}
