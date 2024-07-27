use bevy::prelude::*;

#[derive(Debug, Component, Reflect)]
pub struct OngoingCast {
    slot_e: Entity,
    ability_e: Entity,
    cast_timer: Timer,
}

#[derive(Debug)]
pub struct OngoingCastPlugin;

impl Plugin for OngoingCastPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<OngoingCast>();
    }
}
