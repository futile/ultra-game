use std::time::Duration;

use bevy::{ecs::system::SystemParam, prelude::*};
use derive_more::{Display, Error};

use super::{
    ability::{
        AbilityCastTime, AbilityCooldown, AbilityId, AbilitySlotRequirement, CastFailureReason,
        PerformAbility,
    },
    ability_slots::AbilitySlot,
    commands::{GameCommand, GameCommandKind},
    fight::{FightInterface, FightStatus},
    ongoing_cast::{OngoingCast, OngoingCastFinishedSuccessfully, OngoingCastInterface},
};
use crate::{
    PerUpdateSet,
    abilities::AbilityInterface,
    game_logic::{ability::Ability, cooldown::Cooldown},
};

#[derive(SystemParam)]
pub struct AbilityCastingInterface<'w, 's> {
    abilities: Query<'w, 's, &'static Ability>,
    ability_slots: Query<'w, 's, &'static AbilitySlot>,
    ability_slot_requirements: Query<'w, 's, &'static AbilitySlotRequirement>,
    has_cooldown: Query<'w, 's, Has<Cooldown>>,
    pub ability_interface: AbilityInterface<'w, 's>,
    pub fight_interface: FightInterface<'w, 's>,
    pub ongoing_cast_interface: OngoingCastInterface<'w, 's>,
}

/// Represents the usage of an ability
#[derive(Debug, Clone, Component, Reflect)]
pub struct UseAbility {
    pub caster_e: Entity,
    pub slot_e: Entity,
    pub ability_e: Entity,
    pub target: Option<Entity>,
    pub fight_e: Entity,
}

#[derive(Debug, Display, Error)]
pub enum InvalidCastReason {
    FightEnded,
    AbilityOrSlotOnCooldown,
    CantUseSlot,
}

impl<'w, 's> AbilityCastingInterface<'w, 's> {
    /// Checks if the cast request matches the given ability ID
    pub fn is_matching_cast(&self, cast: &UseAbility, id: &AbilityId) -> bool {
        let ability = self.abilities.get(cast.ability_e).unwrap();
        ability.id == *id
    }

    // TODO: Need to get this logic unified/de-duplicated, also for AI (and UI maybe?). Probs after
    // ability refactoring.
    /// Validates if the cast request is valid (fight ongoing, slot compatibility)
    pub fn is_valid_cast(&self, cast: &UseAbility) -> Result<(), InvalidCastReason> {
        match self.fight_interface.get_fight_status(cast.fight_e) {
            FightStatus::Ongoing => (),
            FightStatus::Ended => {
                return Err(InvalidCastReason::FightEnded);
            }
        };

        // Check cooldowns
        if self
            .has_cooldown
            .iter_many([cast.ability_e, cast.slot_e])
            .any(|has_cd| has_cd)
        {
            return Err(InvalidCastReason::AbilityOrSlotOnCooldown);
        }

        // Check slot requirement
        if let Ok(requirement) = self.ability_slot_requirements.get(cast.ability_e) {
            let slot = self.ability_slots.get(cast.slot_e).unwrap();
            if requirement.0 != slot.tpe {
                return Err(InvalidCastReason::CantUseSlot);
            }
        }

        Ok(())
    }

    // TODO: probably merge `use_slot()` and `start_cast()` into an fn like `use_ability()`,
    // which takes an `(&)UseAbility` and resolves the casting logic. Or this might be exactly
    // backwards, because ability impls (those triggered by `UseAbility`) call this, and we don't
    // want to loop.

    /// Uses a slot for an instant ability, interrupting any ongoing cast on it
    pub fn use_slot(&mut self, slot_e: Entity) {
        self.interrupt_cast_on_slot(slot_e);
    }

    /// Starts a cast on a slot, automatically interrupting any existing cast on the same slot
    pub fn start_cast(&mut self, slot_e: Entity, cast: OngoingCast) -> Entity {
        // The OngoingCast system will automatically handle interruption when we create the new cast
        // (see on_add_ongoing_cast observer in ongoing_cast.rs)
        self.ongoing_cast_interface.start_new_cast(slot_e, cast)
    }

    /// Interrupts any ongoing cast on the specified slot (low-level method)
    fn interrupt_cast_on_slot(&mut self, slot_e: Entity) {
        self.ongoing_cast_interface.cancel_ongoing_cast(slot_e);
    }
}

/// Spawns a CastRequest entity for each UseAbility command
fn request_ability_cast(mut commands: Commands, mut game_commands: MessageReader<GameCommand>) {
    for command in game_commands.read() {
        #[expect(irrefutable_let_patterns, reason = "We only have UseAbility currently")]
        if let GameCommandKind::UseAbility(use_ability) = &command.kind {
            commands.spawn(use_ability.clone());
        }
    }
}

/// Checks if the ability is on cooldown
fn check_ability_cooldowns(
    cast_requests: Query<(Entity, &UseAbility), Without<CastFailureReason>>,
    has_cooldown: Query<Has<Cooldown>>,
    mut commands: Commands,
) {
    for (req_e, use_ability) in cast_requests.iter() {
        if has_cooldown.get(use_ability.ability_e).unwrap_or(false) {
            commands
                .entity(req_e)
                .insert(CastFailureReason::AbilityCooldown);
        }
    }
}

fn check_slot_cooldowns(
    cast_requests: Query<(Entity, &UseAbility), Without<CastFailureReason>>,
    has_cooldown: Query<Has<Cooldown>>,
    mut commands: Commands,
) {
    for (req_e, use_ability) in cast_requests.iter() {
        if has_cooldown.get(use_ability.slot_e).unwrap_or(false) {
            commands
                .entity(req_e)
                .insert(CastFailureReason::SlotCooldown);
        }
    }
}

fn check_slot_requirements(
    cast_requests: Query<(Entity, &UseAbility), Without<CastFailureReason>>,
    abilities: Query<&AbilitySlotRequirement>,
    slots: Query<&AbilitySlot>,
    mut commands: Commands,
) {
    for (req_e, use_ability) in cast_requests.iter() {
        let Ok(requirement) = abilities.get(use_ability.ability_e) else {
            // TODO: Can we enforce this at compile time somehow?
            error!(
                "Ability {} has no slot requirement but uses a slot. UseAbility: {use_ability:?}",
                use_ability.ability_e
            );
            continue;
        };

        let Ok(slot) = slots.get(use_ability.slot_e) else {
            error!(
                "Slot entity {} does not have an AbilitySlot component in UseAbility: {use_ability:?}",
                use_ability.slot_e
            );
            commands
                .entity(req_e)
                .insert(CastFailureReason::SlotRequirement);
            continue;
        };

        if requirement.0 != slot.tpe {
            commands
                .entity(req_e)
                .insert(CastFailureReason::SlotRequirement);
        }
    }
}

fn process_valid_casts(
    cast_requests: Query<(Entity, &UseAbility), Without<CastFailureReason>>,
    mut ability_casting_interface: AbilityCastingInterface,
    ability_cast_times: Query<&AbilityCastTime>,
    mut commands: Commands,
) {
    for (req_e, use_ability) in cast_requests.iter() {
        // Use the slot (interrupts, applies slot on-use cooldown)
        ability_casting_interface.use_slot(use_ability.slot_e);

        let cast_duration = ability_cast_times
            .get(use_ability.ability_e)
            .map(|ct| ct.0)
            .unwrap_or(Duration::ZERO);

        let ongoing_cast = OngoingCast {
            ability_e: use_ability.ability_e,
            target: use_ability.target,
            cast_timer: Timer::new(cast_duration, TimerMode::Once),
        };

        // Start the cast (spawns OngoingCast entity attached to slot)
        ability_casting_interface.start_cast(use_ability.slot_e, ongoing_cast);

        // Despawn the request
        commands.entity(req_e).despawn();
    }
}

fn cleanup_failed_casts(
    cast_requests: Query<Entity, With<CastFailureReason>>,
    mut commands: Commands,
) {
    for req_e in cast_requests.iter() {
        // TODO: Emit UI error events here
        commands.entity(req_e).despawn();
    }
}

/// Observer that applies slot cooldowns when ongoing casts finish successfully
fn apply_slot_cooldown_on_cast_finish(
    trigger: On<OngoingCastFinishedSuccessfully>,
    ability_slots: Query<&AbilitySlot>,
    mut commands: Commands,
) {
    let slot_e = trigger.event().slot_entity;

    // Apply slot-defined cooldown if present
    if let Ok(slot) = ability_slots.get(slot_e)
        && let Some(cooldown_duration) = slot.on_use_cooldown
    {
        commands
            .entity(slot_e)
            .insert(Cooldown::new(cooldown_duration));
    }
}

/// Observer that applies ability cooldowns when ongoing casts finish successfully
fn apply_ability_cooldown_on_cast_finish(
    trigger: On<OngoingCastFinishedSuccessfully>,
    ability_cooldowns: Query<&AbilityCooldown>,
    mut commands: Commands,
) {
    let ability_e = trigger.event().ability_entity;

    if let Ok(cooldown) = ability_cooldowns.get(ability_e) {
        commands
            .entity(ability_e)
            .insert(Cooldown::new(cooldown.duration));
    }
}

/// Observer that triggers PerformAbility when OngoingCast finishes
fn trigger_perform_ability(trigger: On<OngoingCastFinishedSuccessfully>, mut commands: Commands) {
    let event = trigger.event();
    commands.trigger(PerformAbility {
        ability_entity: event.ability_entity,
        target: event.cast_target,
        slot: event.slot_entity,
    });
}

#[derive(Debug)]
pub struct AbilityCastingPlugin;

impl Plugin for AbilityCastingPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(apply_slot_cooldown_on_cast_finish)
            .add_observer(apply_ability_cooldown_on_cast_finish)
            .add_observer(trigger_perform_ability)
            .register_type::<UseAbility>()
            .add_systems(
                Update,
                (
                    request_ability_cast,
                    (
                        check_ability_cooldowns,
                        check_slot_cooldowns,
                        check_slot_requirements,
                    ),
                    (process_valid_casts, cleanup_failed_casts),
                )
                    .chain()
                    .in_set(PerUpdateSet::CommandResolution),
            );
    }
}
