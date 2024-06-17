use bevy::prelude::*;

use crate::PerUpdateSet;

#[derive(Debug, Clone, Component, Reflect, PartialEq)]
pub struct DamageInstance {
    pub source: Option<Entity>,
    pub target: Entity,
    pub amount: f64,
}

#[derive(Debug, Event)]
pub struct DealDamage(pub DamageInstance);

fn damage_resolution_system(mut deal_damage_events: EventReader<DealDamage>) {
    for deal_damage_event in deal_damage_events.read() {
        println!("{deal_damage_event:?}");

        // silence "field not read"-warning
        // TODO: actually use the damage event to do something..
        let _damage = &deal_damage_event.0;
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
