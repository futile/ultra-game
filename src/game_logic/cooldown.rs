use std::time::Duration;

use bevy::{ecs::system::SystemParam, prelude::*};

use crate::{
    PerUpdateSet,
    game_logic::{
        ability::AbilityId,
        ability_slots::AbilitySlot,
        fight::{FightInterface, FightTime},
    },
    utils::holds_held::Held,
};

#[derive(Debug, Component, Reflect)]
pub struct Cooldown {
    cooldown_timer: Timer,
}

impl Cooldown {
    /// Starts a new [`Cooldown`] that should be on the same entity as an [`AbilityId`] or
    /// [`AbilitySlot`]
    pub fn new(cooldown_duration: Duration) -> Cooldown {
        Cooldown {
            cooldown_timer: Timer::new(cooldown_duration, TimerMode::Once),
        }
    }

    /// Returns the duration for which this [`Cooldown`] is still running
    pub fn remaining_cooldown(&self) -> Duration {
        self.cooldown_timer.remaining()
    }
}

fn tick_cooldowns(
    cooldowns: Query<(Entity, &mut Cooldown), Without<FightTime>>,
    time: Res<Time>,
    cooldown_interface: CooldownInterface,
    fight_interface: FightInterface,
    mut commands: Commands,
) {
    let delta = time.delta();

    for (e, mut cooldown) in cooldowns {
        let character = cooldown_interface.find_character_of_cooldown(e).unwrap();
        let fight_e = fight_interface.get_fight_of_entity(character);

        if fight_interface.is_fight_paused(fight_e) {
            continue;
        }

        if cooldown.cooldown_timer.is_finished() {
            warn!(
                "finished Cooldown exists on entity '{e}', but should already be removed. removing it now."
            );
        }

        cooldown.cooldown_timer.tick(delta);

        if cooldown.cooldown_timer.is_finished() {
            commands.entity(e).remove::<Cooldown>();
        }
    }
}

#[derive(SystemParam)]
pub struct CooldownInterface<'w, 's> {
    held_ability_ids: Query<'w, 's, &'static Held<AbilityId>>,
    held_ability_slots: Query<'w, 's, &'static Held<AbilitySlot>>,
}

impl<'w, 's> CooldownInterface<'w, 's> {
    /// `entity` should have a [`Cooldown`] and be either an [`AbilityId`] or [`AbilitySlot`]
    pub fn find_character_of_cooldown(&self, entity: Entity) -> Option<Entity> {
        // First try to find if this entity has Held<AbilityId>,
        // then try to find if this entity has Held<AbilitySlot>
        self.held_ability_ids
            .related(entity)
            .or(self.held_ability_slots.related(entity))
    }
}

#[derive(Debug)]
pub struct CooldownPlugin;

impl Plugin for CooldownPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Cooldown>().add_systems(
            FixedUpdate,
            tick_cooldowns.in_set(PerUpdateSet::LogicUpdate),
        );
    }
}
