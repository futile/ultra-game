use bevy::{
    ecs::{component::HookContext, system::SystemParam, world::DeferredWorld},
    prelude::*,
};

use super::fight::FightInterface;
use crate::{PerUpdateSet, game_logic::ability_slots::AbilitySlot, utils::holds_held::Held};

// TODO:
// * Maybe add `target_e` here? some other way to find out *what* is happening? -> Probably as I
//   need during development.
// * When multiple `OngoingCast`s per Slot should be supported, add a Relationship here
#[derive(Debug, Component, Reflect)]
#[component(on_replace = on_replace_ongoing_cast)]
pub struct OngoingCast {
    pub ability_e: Entity,
    pub cast_timer: Timer,
}

#[derive(Debug, Reflect, Event)]
pub struct OngoingCastFinishedSuccessfully;

#[derive(Debug, Reflect, Event)]
pub struct OngoingCastAborted;

#[derive(SystemParam)]
pub struct OngoingCastInterface<'w, 's> {
    ongoing_casts: Query<'w, 's, &'static OngoingCast>,
    commands: Commands<'w, 's>,
}

impl<'w, 's> OngoingCastInterface<'w, 's> {
    pub fn start_new_cast(&mut self, slot_e: Entity, cast: OngoingCast) -> Entity {
        self.commands.entity(slot_e).insert(cast).id()
    }

    /// Retrieves the [`OngoingCast`] for an `AbilitySlot` entity, if it has one
    pub fn get_ongoing_cast(&self, slot_e: Entity) -> Option<&OngoingCast> {
        self.ongoing_casts.get(slot_e).ok()
    }

    /// Cancels any ongoing cast on the specified entity (currently `AbilitySlot` entities)
    pub fn cancel_ongoing_cast(&mut self, slot_e: Entity) {
        self.commands.entity(slot_e).remove::<OngoingCast>();
    }
}

fn tick_ongoing_casts(
    mut ongoing_casts: Query<(Entity, &mut OngoingCast, &Held<AbilitySlot>)>,
    fight_interface: FightInterface,
    time: Res<Time>,
    mut commands: Commands,
) {
    for (slot_e, mut ongoing_cast, held_ability_slot) in &mut ongoing_casts {
        let slot_holder_e = held_ability_slot.held_by;

        if fight_interface.is_fight_paused(fight_interface.get_fight_of_entity(slot_holder_e)) {
            continue;
        }

        assert!(!ongoing_cast.cast_timer.finished());

        ongoing_cast.cast_timer.tick(time.delta());

        if ongoing_cast.cast_timer.just_finished() {
            commands.trigger_targets(OngoingCastFinishedSuccessfully, slot_e);
            commands.entity(slot_e).remove::<OngoingCast>();
        }
    }
}

fn on_replace_ongoing_cast(mut world: DeferredWorld, hook_context: HookContext) {
    let ongoing_cast = world.get::<OngoingCast>(hook_context.entity).unwrap();

    if !ongoing_cast.cast_timer.finished() {
        // maybe fire an event or sth. -- need to make sure the `OngoingCast` isn't despawned
        // while the event is still being handled..
        // -> does indeed remove the `OngoingCast` before the event is being handled.
        // TODO: maybe consumers should also listen for `OnReplaced<OngoingCast>`, instead of this
        // event? and then have a method like `OngoingCast::finished_successfully()`, that will
        // return `false` (or `Aborted` etc.) in this case.
        world.trigger_targets(OngoingCastAborted, hook_context.entity);
    }
}

#[derive(Debug)]
pub struct OngoingCastPlugin;

impl Plugin for OngoingCastPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<OngoingCast>().add_systems(
            FixedUpdate,
            tick_ongoing_casts.in_set(PerUpdateSet::LogicUpdate),
        );
    }
}
