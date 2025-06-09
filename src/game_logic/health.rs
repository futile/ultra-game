use bevy::{ecs::system::SystemParam, prelude::*};

#[derive(Debug, Clone, Component, Reflect)]
pub struct Health {
    current: f64,
    max: f64,
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

    pub fn current(&self) -> f64 {
        self.current
    }

    pub fn max(&self) -> f64 {
        self.max
    }
}

#[derive(Debug)]
pub struct AlreadyDeadError;

#[derive(Debug, Event, PartialEq, Eq)]
pub enum LivenessChangeEvent {
    EntityDied { which: Entity },
}

#[derive(SystemParam)]
pub struct HealthInterface<'w, 's> {
    healths: Query<'w, 's, &'static mut Health>,
    liveness_events: EventWriter<'w, LivenessChangeEvent>,
}

impl<'w, 's> HealthInterface<'w, 's> {
    pub fn lose_hp(&mut self, target: Entity, amount: f64) -> Result<(), AlreadyDeadError> {
        let mut target_health = self.healths.get_mut(target).unwrap();

        if target_health.is_alive() {
            target_health.current -= amount;

            if target_health.is_dead() {
                self.liveness_events
                    .write(LivenessChangeEvent::EntityDied { which: target });
            }

            Ok(())
        } else {
            Err(AlreadyDeadError)
        }
    }

    pub fn healths(&self) -> Query<'_, 's, &'static Health> {
        self.healths.as_readonly()
    }
}

pub struct HealthInterfacePlugin;

impl Plugin for HealthInterfacePlugin {
    fn build(&self, app: &mut App) {
        // from https://github.com/jakobhellermann/bevy-inspector-egui/discussions/130
        app.register_type::<Health>()
            .add_event::<LivenessChangeEvent>();
    }
}
