use std::time::Duration;

use bevy::prelude::*;

use crate::{
    abilities::weapon_attack,
    game_logic::{
        ability::Ability,
        ability_slots::{AbilitySlot, AbilitySlotType},
        faction::Faction,
        fight::FightBundle,
        health::Health,
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

pub fn spawn_test_fight(app: &mut App) -> TestFightEntities {
    let mut commands = app.world_mut().commands();

    let slot_e = commands
        .spawn(AbilitySlot {
            tpe: AbilitySlotType::WeaponAttack,
            on_use_cooldown: Some(Duration::from_secs(1)),
        })
        .id();

    let ability_e = weapon_attack::spawn_weapon_attack(&mut commands);

    let caster_e = commands
        .spawn((
            Health::new(100.0),
            Faction::Player,
            Name::new("Player Character"),
        ))
        .add_one_related::<Held<AbilitySlot>>(slot_e)
        .add_one_related::<Held<Ability>>(ability_e)
        .id();

    let enemy_ability_e = weapon_attack::spawn_weapon_attack(&mut commands);

    let enemy_e = commands
        .spawn((Name::new("The Enemy"), Health::new(100.0), Faction::Enemy))
        .with_related_entities::<Held<AbilitySlot>>(|commands| {
            commands.spawn(AbilitySlot {
                tpe: AbilitySlotType::WeaponAttack,
                on_use_cooldown: Some(Duration::from_secs(1)),
            });
        })
        .add_one_related::<Held<Ability>>(enemy_ability_e)
        .id();

    let fight_e = commands
        .spawn((FightBundle::new(), Name::new("The Fight")))
        .add_children(&[caster_e, enemy_e])
        .id();

    // Flush commands
    app.world_mut().flush();

    TestFightEntities {
        fight_e,
        caster_e,
        slot_e,
        ability_e,
        enemy_e,
    }
}
