use bevy::{ecs::system::SystemParam, prelude::*};

use crate::{
    PerUpdateSet,
    game_logic::{ability::AbilityId, ability_slots::AbilitySlot},
    utils::holds_held::Held,
};

#[derive(Debug, Component, Reflect)]
pub struct Cooldown {
    cooldown_timer: Timer,
}

fn tick_cooldowns(
    cooldowns: Query<(Entity, &mut Cooldown)>,
    time: Res<Time>,
    mut commands: Commands,
) {
    let delta = time.delta();

    for (e, mut cooldown) in cooldowns {
        // TODO: retrieve fight of `e` and check if it's paused -> if so, skip

        if cooldown.cooldown_timer.finished() {
            warn!(
                "finished Cooldown exists on entity '{e}', but should already be removed. removing it now."
            );
        } else {
            cooldown.cooldown_timer.tick(delta);
        }

        if cooldown.cooldown_timer.finished() {
            commands.entity(e).remove::<Cooldown>();
        }
    }
}

#[derive(SystemParam)]
pub struct CooldownInterface<'w, 's> {
    cooldowns: Query<'w, 's, (Entity, &'static Cooldown)>,
    held_ability_ids: Query<'w, 's, (Entity, &'static Held<AbilityId>)>,
    held_ability_slots: Query<'w, 's, (Entity, &'static Held<AbilitySlot>)>,
    commands: Commands<'w, 's>,
}

impl<'w, 's> CooldownInterface<'w, 's> {
    // TODO: start_cooldown(&mut self, entity, Duration(Cooldown?), (replace_existing? maybe later,
    // when needed))

    // TODO: is_on_cooldown(&self, entity) -> bool

    // TODO: remaining_cooldown(&self, entity) -> Option<Duration>

    // TODO: find_character_of_cooldown(&self, entity) -> Option<Entity>
    // will traverse `Held<AbilityId>` or `Held<AbilitySlot>` upwards

    // maybe, but maybe callers could/should just use FightInterface for this step,
    // makes everything a bit more transparent too
    // TODO: find_fight_of_cooldown(&self, entity) -> Option<Entity>
    // will traverse (`Held<AbilityId>` or `Held<AbilitySlot>`) then (Parent)
    // (i.e., from AbilitySlot -> "holder")
}

#[derive(Debug)]
pub struct CooldownPlugin;

impl Plugin for CooldownPlugin {
    fn build(&self, app: &mut App) {
        // examples:
        // app.add_systems(Startup, add_to_ability_catalog)
        app.register_type::<Cooldown>().add_systems(
            FixedUpdate,
            tick_cooldowns.in_set(PerUpdateSet::LogicUpdate),
        );
    }
}
