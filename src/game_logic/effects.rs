use bevy::prelude::*;

#[derive(Debug, Component, Reflect)]
pub struct HasEffects {
    pub holder: Entity,
}

#[derive(Debug)]
pub struct EffectsPlugin;

impl Plugin for EffectsPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<HasEffects>();
    }
}
