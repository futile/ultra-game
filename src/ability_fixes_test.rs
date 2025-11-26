#[cfg(test)]
mod tests {
    use std::{assert_matches::assert_matches, time::Duration};

    use bevy::{prelude::*, time::TimeUpdateStrategy};

    use crate::{
        abilities::{
            AbilityCatalog,
            needling_hex::{NeedlingHexEffect, NeedlingHexPlugin},
            weapon_attack::WeaponAttackPlugin,
        },
        game_logic::{
            ability::{Ability, AbilityCastTime, AbilityCooldown, AbilityId},
            ability_casting::{AbilityCastingPlugin, UseAbility},
            ability_slots::{AbilitySlot, AbilitySlotType},
            commands::{CommandsPlugin, GameCommand, GameCommandKind},
            cooldown::Cooldown,
            damage_resolution::{DamageResolutionPlugin, DealDamage},
            effects::HasEffects,
            faction::Faction,
            fight::{FightBundle, FightPlugin, FightTime},
            health::Health,
            ongoing_cast::{OngoingCast, OngoingCastFinishedSuccessfully, OngoingCastPlugin},
        },
        utils::holds_held::Held,
    };

    pub struct TestFightEntities {
        pub fight_e: Entity,
        pub caster_e: Entity,
        pub slot_e: Entity,
        pub ability_e: Entity,
        pub enemy_e: Entity,
    }

    fn spawn_test_fight(app: &mut App) -> TestFightEntities {
        let mut commands = app.world_mut().commands();
        let fight_e = commands.spawn(FightBundle::new()).id();

        // Spawn caster
        let caster_e = commands.spawn_empty().id();
        commands.entity(fight_e).add_child(caster_e);

        // Spawn slot
        let slot_e = commands
            .spawn(AbilitySlot {
                tpe: AbilitySlotType::WeaponAttack,
                on_use_cooldown: None,
            })
            .id();

        // Attach slot to caster (Held<AbilitySlot>)
        commands.entity(slot_e).insert(Held::<AbilitySlot> {
            held_by: caster_e,
            _phantom_t: std::marker::PhantomData,
        });

        // Spawn ability
        let ability_e = commands
            .spawn((
                AbilityId::Attack,
                AbilityCastTime(Duration::ZERO),
                Held::<Ability> {
                    held_by: caster_e,
                    _phantom_t: std::marker::PhantomData,
                },
            ))
            .id();

        // Spawn enemy
        let enemy_e = commands
            .spawn((Name::new("Test Enemy"), Health::new(100.0), Faction::Enemy))
            .id();
        commands.entity(fight_e).add_child(enemy_e);

        // Flush commands
        app.update();

        TestFightEntities {
            fight_e,
            caster_e,
            slot_e,
            ability_e,
            enemy_e,
        }
    }

    #[test]
    fn test_fight_timer_starts_on_ability_use() {
        let mut app = App::new();
        app.init_resource::<AbilityCatalog>()
            .add_plugins(MinimalPlugins)
            .add_plugins(FightPlugin)
            .add_plugins(CommandsPlugin)
            .add_plugins(AbilityCastingPlugin)
            .add_plugins(OngoingCastPlugin);

        let TestFightEntities {
            fight_e,
            caster_e,
            slot_e,
            ability_e,
            enemy_e: _,
        } = spawn_test_fight(&mut app);

        // Verify timer paused
        let fight_time = app.world().get::<FightTime>(fight_e).unwrap();
        assert!(fight_time.is_paused());

        app.world_mut()
            .write_message(GameCommand::new_from_user(GameCommandKind::UseAbility(
                UseAbility {
                    caster_e,
                    slot_e,
                    ability_e,
                    target: None,
                    fight_e,
                },
            )));

        app.update();

        // Verify timer unpaused
        let fight_time = app.world().get::<FightTime>(fight_e).unwrap();
        assert!(!fight_time.is_paused(), "Fight timer should be unpaused");
    }

    #[test]
    fn test_ability_cooldown_applied_on_finish() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_plugins(CommandsPlugin)
            .add_plugins(AbilityCastingPlugin)
            .add_plugins(OngoingCastPlugin);

        let ability_e = app
            .world_mut()
            .spawn((
                AbilityId::Attack,
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

        // Verify cooldown applied
        assert!(
            app.world().get::<Cooldown>(ability_e).is_some(),
            "Ability should have Cooldown component"
        );
    }

    #[test]
    fn test_interruption_does_not_apply_cooldown() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_plugins(CommandsPlugin)
            .add_plugins(AbilityCastingPlugin)
            .add_plugins(OngoingCastPlugin);

        let ability_e = app
            .world_mut()
            .spawn((
                AbilityId::Attack,
                AbilityCooldown {
                    duration: Duration::from_secs(5),
                },
            ))
            .id();
        let slot_e = app
            .world_mut()
            .spawn(AbilitySlot {
                tpe: AbilitySlotType::WeaponAttack,
                on_use_cooldown: Some(Duration::from_secs(1)),
            })
            .id();

        // Manually start a cast (as if by system)
        app.world_mut().entity_mut(slot_e).insert(OngoingCast {
            ability_e,
            target: None,
            cast_timer: Timer::from_seconds(1.0, TimerMode::Once),
        });

        // Interrupt it (by starting another cast or calling cancel)
        app.world_mut().entity_mut(slot_e).remove::<OngoingCast>();

        app.update();

        // Verify NO cooldown applied
        assert!(
            app.world().get::<Cooldown>(ability_e).is_none(),
            "Ability should NOT have Cooldown component"
        );
        assert!(
            app.world().get::<Cooldown>(slot_e).is_none(),
            "Slot should NOT have Cooldown component"
        );
    }

    #[test]
    fn test_cooldown_ticks_and_removes() {
        use crate::game_logic::cooldown::{Cooldown, CooldownPlugin};

        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
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

        // Automatically advance time by 55ms (little bit more than half the cooldown) on each
        // update
        // See: https://docs.rs/bevy/latest/bevy/time/enum.TimeUpdateStrategy.html
        // See: https://taintedcoders.com/bevy/apps#schedules
        // See: https://taintedcoders.com/bevy/how-to/fixed-timestep
        app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(
            55,
        )));

        // Advance time by 55ms (little bit more than half the cooldown)
        app.update();

        // Cooldown should still exist
        let cooldown = app.world().get::<Cooldown>(ability_e);
        assert!(cooldown.is_some(), "Cooldown should still exist after 55ms");
        assert!(
            cooldown.unwrap().remaining_cooldown() > Duration::ZERO,
            "Cooldown should have time remaining"
        );

        // Advance time by another 55ms (total 110ms, past the 100ms cooldown)
        app.update();

        // Cooldown should be removed
        assert_matches!(
            app.world().get::<Cooldown>(ability_e),
            None,
            "Cooldown should be removed after finishing"
        );
    }

    #[test]
    fn test_wrong_effects() {
        let mut app = App::new();
        app.init_resource::<AbilityCatalog>()
            .add_plugins(MinimalPlugins)
            .add_plugins(CommandsPlugin)
            .add_plugins(AbilityCastingPlugin)
            .add_plugins(OngoingCastPlugin)
            .add_plugins(WeaponAttackPlugin)
            .add_plugins(NeedlingHexPlugin)
            .add_plugins(FightPlugin)
            .add_plugins(DamageResolutionPlugin);

        let TestFightEntities {
            fight_e: _,
            caster_e: _,
            slot_e,
            ability_e: attack_ability_e,
            enemy_e,
        } = spawn_test_fight(&mut app);

        // 1. Test Attack
        app.world_mut().trigger(OngoingCastFinishedSuccessfully {
            slot_entity: slot_e,
            ability_entity: attack_ability_e,
            cast_target: Some(enemy_e),
        });

        app.update();

        // Verify DealDamage event
        let events = app.world().resource::<Messages<DealDamage>>();
        assert!(!events.is_empty(), "Should have DealDamage event");

        // Verify NO NeedlingHexEffect
        let has_effects = app.world().get::<HasEffects>(enemy_e);
        assert!(has_effects.is_none(), "Target should have NO HasEffects");

        // Clear events
        app.world_mut()
            .resource_mut::<Messages<DealDamage>>()
            .clear();

        // 2. Test Needling Hex
        let hex_ability_e = app.world_mut().spawn(AbilityId::NeedlingHex).id();

        app.world_mut().trigger(OngoingCastFinishedSuccessfully {
            slot_entity: slot_e,
            ability_entity: hex_ability_e,
            cast_target: Some(enemy_e),
        });

        app.update();
        app.update();

        // Verify NeedlingHexEffect
        // NeedlingHexEffect is not on the target, but on a child of the holder.
        // We check if HasEffects is present, and if we can find the effect.

        let has_effects = app.world().get::<HasEffects>(enemy_e);
        assert!(has_effects.is_some(), "Target should have HasEffects");

        let holder_e = has_effects.unwrap().holder();
        let children = app.world().get::<Children>(holder_e);
        assert!(children.is_some(), "Holder should have children");

        let found = children
            .unwrap()
            .iter()
            .any(|child| app.world().get::<NeedlingHexEffect>(child).is_some());
        assert!(
            found,
            "Should find NeedlingHexEffect on a child of the holder"
        );
    }
}
