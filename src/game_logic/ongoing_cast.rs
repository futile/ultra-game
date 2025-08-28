use bevy::{ecs::system::SystemParam, prelude::*};

use super::fight::FightInterface;
use crate::PerUpdateSet;

// TODO: Maybe add `target_e` here? some other way to find out *what* is happening?
// Probably as I need during development.
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

#[derive(Debug, Reflect, Event)]
pub struct OngoingCastFinishedSuccessfully;

#[derive(Debug, Reflect, Event)]
pub struct OngoingCastAborted;

#[derive(SystemParam)]
pub struct OngoingCastInterface<'w, 's> {
    has_ongoing_casts: Query<'w, 's, &'static HasOngoingCast>,
    ongoing_casts: Query<'w, 's, &'static OngoingCast>,
    commands: Commands<'w, 's>,
}

impl<'w, 's> OngoingCastInterface<'w, 's> {
    pub fn start_new_cast(&mut self, cast: OngoingCast) -> Entity {
        self.commands.spawn(cast).id()
    }

    /// Retrieves the [`OngoingCast`] for an entity that has a [`HasOngoingCast`].
    pub fn get_ongoing_cast(&self, entity: Entity) -> Option<&OngoingCast> {
        self.has_ongoing_casts
            .get(entity)
            .ok()
            .and_then(|hoc| self.ongoing_casts.get(hoc.ongoing_cast_e).ok())
    }

    /// Cancels any ongoing cast on the specified entity (currently Slot entities)
    pub fn cancel_ongoing_cast(&mut self, entity: Entity) {
        if let Ok(has_ongoing_cast) = self.has_ongoing_casts.get(entity) {
            // Despawn the ongoing cast entity, which will trigger the removal observer
            self.commands
                .entity(has_ongoing_cast.ongoing_cast_e)
                .despawn();
        }
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
            commands.trigger_targets(OngoingCastFinishedSuccessfully, e);
            commands.entity(e).despawn();
        }
    }
}

fn on_add_ongoing_cast(
    trigger: Trigger<OnAdd, OngoingCast>,
    ongoing_casts: Query<&OngoingCast>,
    has_ongoing_casts: Query<&HasOngoingCast>,
    mut commands: Commands,
) {
    let ongoing_cast = ongoing_casts.get(trigger.target()).unwrap();

    // if this is true, then we are "overriding" an ongoing cast.
    if let Ok(previous_ongoing_cast) = has_ongoing_casts.get(ongoing_cast.slot_e) {
        // maybe fire an event or sth. -- need to make sure the `ongoing_cast_e` isn't despawned
        // while the event is still being handled..
        // TODO: test if this works as expected (:
        commands.trigger_targets(OngoingCastAborted, previous_ongoing_cast.ongoing_cast_e);

        commands
            .entity(previous_ongoing_cast.ongoing_cast_e)
            .despawn();
    }

    // track [`HasOngoingCast`] on the slot entity
    commands.entity(ongoing_cast.slot_e).insert(HasOngoingCast {
        ongoing_cast_e: trigger.target(),
    });
}

fn on_remove_ongoing_cast(
    trigger: Trigger<OnRemove, OngoingCast>,
    ongoing_casts: Query<&OngoingCast>,
    has_ongoing_casts: Query<&HasOngoingCast>,
    mut commands: Commands,
) {
    let ongoing_cast = ongoing_casts.get(trigger.target()).unwrap();

    // remove HasOngoingCast from the slot entity
    let slot_e = ongoing_cast.slot_e;

    // if another OngoingCast has overriden the one we are despawning, these will not be equal.
    if has_ongoing_casts.get(slot_e).unwrap().ongoing_cast_e == trigger.target() {
        commands.entity(slot_e).remove::<HasOngoingCast>();
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
            .add_observer(on_add_ongoing_cast)
            .add_observer(on_remove_ongoing_cast);
    }
}
