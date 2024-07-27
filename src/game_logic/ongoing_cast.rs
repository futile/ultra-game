use bevy::{ecs::system::SystemParam, prelude::*};

use super::fight::FightInterface;
use crate::PerUpdateSet;

#[derive(Debug, Component, Reflect)]
pub struct OngoingCast {
    pub fight_e: Entity,
    pub slot_e: Entity,
    pub ability_e: Entity,
    pub cast_timer: Timer,
}

#[derive(Debug, Component, Reflect)]
pub struct HasOngoingCast {
    ongoing_cast_e: Entity,
}

#[derive(SystemParam)]
pub struct OngoingCastInterface<'w, 's> {
    commands: Commands<'w, 's>,
}

impl<'w, 's> OngoingCastInterface<'w, 's> {
    pub fn start_new_cast(&mut self, cast: OngoingCast) {
        self.commands.spawn(cast);
    }
}

fn tick_ongoing_casts(
    mut ongoing_casts: Query<(Entity, &mut OngoingCast)>,
    fight_interface: FightInterface,
    time: Res<Time>,
    mut commands: Commands,
) {
    for (e, mut ongoing_cast) in &mut ongoing_casts {
        if fight_interface.is_fight_paused(ongoing_cast.fight_e) {
            continue;
        }

        assert!(!ongoing_cast.cast_timer.finished());

        ongoing_cast.cast_timer.tick(time.delta());

        if ongoing_cast.cast_timer.just_finished() {
            commands.entity(e).despawn_recursive();

            // TODO: Create (and fire) event for this successfully finished cast
            // (maybe before despawning, might matter, because the cast will/should still exist)
        }
    }
}

fn on_add_ongoing_cast(
    trigger: Trigger<OnAdd, OngoingCast>,
    ongoing_casts: Query<&OngoingCast>,
    has_ongoing_casts: Query<&HasOngoingCast>,
    mut commands: Commands,
) {
    let ongoing_cast = ongoing_casts.get(trigger.entity()).unwrap();

    for e in [ongoing_cast.slot_e, ongoing_cast.ability_e] {
        assert!(!has_ongoing_casts.contains(e));

        commands.entity(e).insert(HasOngoingCast {
            ongoing_cast_e: trigger.entity(),
        });
    }
}

fn on_remove_ongoing_cast(
    trigger: Trigger<OnRemove, OngoingCast>,
    ongoing_casts: Query<&OngoingCast>,
    has_ongoing_casts: Query<&HasOngoingCast>,
    mut commands: Commands,
) {
    let ongoing_cast = ongoing_casts.get(trigger.entity()).unwrap();

    for e in [ongoing_cast.slot_e, ongoing_cast.ability_e] {
        assert_eq!(
            has_ongoing_casts.get(e).unwrap().ongoing_cast_e,
            trigger.entity()
        );

        commands.entity(e).remove::<HasOngoingCast>();
    }
}

#[derive(Debug)]
pub struct OngoingCastPlugin;

impl Plugin for OngoingCastPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<OngoingCast>()
            .register_type::<HasOngoingCast>()
            .add_systems(
                FixedUpdate,
                tick_ongoing_casts.in_set(PerUpdateSet::LogicUpdate),
            )
            .observe(on_add_ongoing_cast)
            .observe(on_remove_ongoing_cast);
    }
}
