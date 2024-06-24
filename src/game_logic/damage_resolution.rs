use bevy::prelude::*;

use super::Health;
use crate::PerUpdateSet;

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
    mut healths: Query<&mut Health>,
) {
    for deal_damage_event in deal_damage_events.read() {
        println!("{deal_damage_event:?}");

        let damage = &deal_damage_event.0;

        let mut target_health = healths.get_mut(damage.target).unwrap();

        if target_health.is_alive() {
            target_health.current -= damage.amount;
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
