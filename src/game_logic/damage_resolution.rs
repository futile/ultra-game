use bevy::prelude::*;

use super::health::HealthInterface;
use crate::{PerUpdateSet, game_logic::health::AlreadyDeadError};

#[derive(Debug, Clone, Component, Reflect, PartialEq)]
pub struct DamageInstance {
    pub source: Option<Entity>,
    pub target: Entity,
    pub amount: f64,
}

#[derive(Debug, Event)]
pub struct DealDamage(pub DamageInstance);

fn damage_resolution_system(
    mut deal_damage_events: EventReader<DealDamage>,
    mut health_interface: HealthInterface,
) {
    for deal_damage_event in deal_damage_events.read() {
        let damage = &deal_damage_event.0;

        let res = health_interface.lose_hp(damage.target, damage.amount);
        match res {
            Ok(()) | Err(AlreadyDeadError) => (),
        }
    }
}

pub struct DamageResolutionPlugin;

impl Plugin for DamageResolutionPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<DamageInstance>()
            .add_event::<DealDamage>()
            .add_systems(
                Update,
                damage_resolution_system.in_set(PerUpdateSet::DamageResolution),
            );
    }
}
