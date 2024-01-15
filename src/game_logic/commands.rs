use bevy::prelude::*;

#[derive(Debug, Event)]
pub struct CastAbility {
    pub caster_e: Entity,
    pub slot_e: Option<Entity>,
    pub ability_e: Entity,
    pub fight_e: Entity,
}

pub struct CommandsPlugin;

impl Plugin for CommandsPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<CastAbility>();
    }
}
