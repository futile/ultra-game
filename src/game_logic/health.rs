use bevy::prelude::*;

#[derive(Debug, Clone, Component, Reflect)]
pub struct Health {
    pub current: f64,
    pub max: f64,
}

impl Health {
    pub fn new(current_and_max: f64) -> Self {
        Self {
            current: current_and_max,
            max: current_and_max,
        }
    }

    pub fn is_alive(&self) -> bool {
        self.current > 0.0f64
    }

    pub fn is_dead(&self) -> bool {
        !self.is_alive()
    }
}

pub struct HealthInterfacePlugin;

impl Plugin for HealthInterfacePlugin {
    fn build(&self, app: &mut App) {
        // from https://github.com/jakobhellermann/bevy-inspector-egui/discussions/130
        app.register_type::<Health>();
    }
}
