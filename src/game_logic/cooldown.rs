use std::time::Duration;

use bevy::{ecs::system::SystemParam, prelude::*};

use crate::{
    PerUpdateSet,
    game_logic::{
        ability::Ability,
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
    /// Starts a new [`Cooldown`] that should be on the same entity as an [`Ability`] or
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
    held_abilities: Query<'w, 's, &'static Held<Ability>>,
    held_ability_slots: Query<'w, 's, &'static Held<AbilitySlot>>,
}

impl<'w, 's> CooldownInterface<'w, 's> {
    /// `entity` should have a [`Cooldown`] and be either an [`Ability`] or [`AbilitySlot`]
    pub fn find_character_of_cooldown(&self, entity: Entity) -> Option<Entity> {
        // First try to find if this entity has Held<Ability>,
        // then try to find if this entity has Held<AbilitySlot>
        self.held_abilities
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

#[cfg(test)]
mod tests {
    use std::{assert_matches::assert_matches, time::Duration};

    use bevy::{log::LogPlugin, prelude::*, time::TimeUpdateStrategy};

    use super::{Cooldown, CooldownPlugin};
    use crate::{
        game_logic::{
            ability::{Ability, AbilityCooldown, AbilityId},
            ability_casting::AbilityCastingPlugin,
            ability_slots::{AbilitySlot, AbilitySlotType},
            commands::CommandsPlugin,
            fight::{FightPlugin, FightTime},
            ongoing_cast::{OngoingCastFinishedSuccessfully, OngoingCastPlugin},
        },
        test_utils::{TestFightEntities, spawn_test_fight},
    };

    #[test]
    fn test_cooldown_applied_after_cast_completion() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_plugins(CommandsPlugin)
            .add_plugins(AbilityCastingPlugin)
            .add_plugins(OngoingCastPlugin);

        let ability_e = app
            .world_mut()
            .spawn((
                Ability {
                    id: AbilityId::WeaponAttack,
                    name: "Attack".into(),
                    description: "Attack".into(),
                },
                AbilityCooldown {
                    duration: Duration::from_secs(5),
                },
            ))
            .id();
        let slot_e = app
            .world_mut()
            .spawn(AbilitySlot {
                tpe: AbilitySlotType::WeaponAttack,
                on_use_cooldown: None,
            })
            .id();

        // Trigger cast finish
        app.world_mut().trigger(OngoingCastFinishedSuccessfully {
            slot_entity: slot_e,
            ability_entity: ability_e,
            cast_target: None,
        });

        app.update();

        // Verify cooldown applied to ability
        assert!(
            app.world().get::<Cooldown>(ability_e).is_some(),
            "Ability should have Cooldown component"
        );

        // Verify no cooldown applied to slot
        assert!(
            app.world().get::<Cooldown>(slot_e).is_none(),
            "Slot should not have Cooldown component"
        );
    }

    #[test]
    fn test_slot_cooldown_applied_after_cast_completion() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_plugins(CommandsPlugin)
            .add_plugins(AbilityCastingPlugin)
            .add_plugins(OngoingCastPlugin);

        let ability_e = app
            .world_mut()
            .spawn(Ability {
                id: AbilityId::WeaponAttack,
                name: "Attack".into(),
                description: "Attack".into(),
            })
            .id();
        let slot_e = app
            .world_mut()
            .spawn(AbilitySlot {
                tpe: AbilitySlotType::WeaponAttack,
                on_use_cooldown: Some(Duration::from_secs(3)),
            })
            .id();

        // Trigger cast finish
        app.world_mut().trigger(OngoingCastFinishedSuccessfully {
            slot_entity: slot_e,
            ability_entity: ability_e,
            cast_target: None,
        });

        app.update();

        // Verify cooldown applied to slot
        assert!(
            app.world().get::<Cooldown>(slot_e).is_some(),
            "Slot should have Cooldown component"
        );

        // Verify no cooldown applied to ability (since AbilityCooldown wasn't set)
        assert!(
            app.world().get::<Cooldown>(ability_e).is_none(),
            "Ability should not have Cooldown component"
        );
    }

    #[test]
    fn test_cooldown_expires_after_duration() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_plugins(LogPlugin::default())
            .add_plugins(CommandsPlugin)
            .add_plugins(FightPlugin)
            .add_plugins(CooldownPlugin);

        let TestFightEntities {
            fight_e,
            caster_e: _,
            slot_e: _,
            ability_e,
            enemy_e: _,
        } = spawn_test_fight(&mut app);

        // Unpause fight so cooldowns can tick
        app.world_mut()
            .get_mut::<FightTime>(fight_e)
            .unwrap()
            .set_paused(false);

        // Manually add a cooldown to the ability entity
        app.world_mut()
            .entity_mut(ability_e)
            .insert(Cooldown::new(Duration::from_millis(100)));

        // Verify cooldown exists
        assert!(
            app.world().get::<Cooldown>(ability_e).is_some(),
            "Ability should have Cooldown component"
        );

        // update once to initialize all systems etc., seems to be required, when testing with
        // manual time.
        app.update();

        // Automatically advance time by 65ms (little bit more than half the cooldown) on each
        // update
        // See: https://docs.rs/bevy/latest/bevy/time/enum.TimeUpdateStrategy.html
        // See: https://taintedcoders.com/bevy/apps#schedules
        // See: https://taintedcoders.com/bevy/how-to/fixed-timestep
        //
        // NOTE: Will round down to multiples of `Time<Fixed>`, see
        // https://docs.rs/bevy/latest/bevy/app/struct.FixedMain.html
        app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(
            65,
        )));

        // Advance time by ~65ms (little bit more than half the cooldown)
        app.update();

        // Cooldown should still exist
        let cooldown = app.world().get::<Cooldown>(ability_e);
        assert!(cooldown.is_some(), "Cooldown should still exist after 55ms");
        assert!(
            cooldown.unwrap().remaining_cooldown() > Duration::ZERO,
            "Cooldown should have time remaining"
        );

        // Advance time by another 65ms (total 130ms, past the 100ms cooldown)
        app.update();

        // Cooldown should be removed
        assert_matches!(
            app.world().get::<Cooldown>(ability_e),
            None,
            "Cooldown should be removed after finishing"
        );
    }
}
