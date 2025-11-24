use std::time::Duration;

use bevy::{ecs::system::RunSystemOnce, prelude::*};
use bevy_inspector_egui::bevy_egui::{
    EguiContext, EguiPrimaryContextPass, PrimaryEguiContext, egui,
};
use big_brain::prelude::*;

use crate::{
    abilities::AbilityCatalog,
    game_logic::{
        ability::{Ability, AbilityId},
        ability_slots::{AbilitySlot, AbilitySlotType},
        ai_behavior::{AttackPlayerAction, CanAttackPlayerScorer},
        faction::Faction,
        fight::{Fight, FightBundle},
        health::Health,
    },
    utils::holds_held::Held,
};

pub struct FightSelectionUiPlugin;

impl Plugin for FightSelectionUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(EguiPrimaryContextPass, render_fight_selection_window);
    }
}

/// Despawns the current fight by finding the Fight entity and recursively despawning
/// its children (player and enemy) and the fight itself.
pub fn despawn_current_fight(mut commands: Commands, fights: Query<Entity, With<Fight>>) {
    // if no fights exist, just return
    if fights.is_empty() {
        return;
    }

    // get the current fight, and expect only one to exist
    let current_fight_e = fights.single().unwrap();

    // Despawn the fight entity and its children (player and enemy)
    commands.entity(current_fight_e).despawn();
}

/// Spawns a basic fight
pub fn spawn_basic_fight(mut commands: Commands, ability_catalog: Res<AbilityCatalog>) {
    let player_character = commands
        .spawn((
            Health::new(100.0),
            Faction::Player,
            Name::new("Player Character"),
        ))
        .with_related_entities::<Held<AbilitySlot>>(|commands| {
            commands.spawn(AbilitySlot {
                tpe: AbilitySlotType::WeaponAttack,
                on_use_cooldown: Some(Duration::from_secs(1)),
            });
            commands.spawn(AbilitySlot {
                tpe: AbilitySlotType::ShieldDefend,
                on_use_cooldown: None,
            });
            commands.spawn(AbilitySlot {
                tpe: AbilitySlotType::Magic,
                on_use_cooldown: Some(Duration::from_secs(2)),
            });
        })
        .id();

    // Spawn abilities for player
    for ability_id in [
        AbilityId::Attack,
        AbilityId::NeedlingHex,
        AbilityId::ChargedStrike,
    ] {
        let ability_e = ability_catalog.spawn(ability_id, &mut commands);
        commands.entity(ability_e).insert(Held::<Ability> {
            held_by: player_character,
            _phantom_t: std::marker::PhantomData,
        });
    }

    let enemy = commands
        .spawn((
            Name::new("The Enemy"),
            Health::new(100.0),
            Faction::Enemy,
            Thinker::build()
                .picker(FirstToScore { threshold: 0.5 })
                .when(CanAttackPlayerScorer, AttackPlayerAction),
        ))
        .with_related_entities::<Held<AbilitySlot>>(|commands| {
            commands.spawn(AbilitySlot {
                tpe: AbilitySlotType::WeaponAttack,
                on_use_cooldown: Some(Duration::from_secs(1)),
            });
        })
        .id();

    // Spawn abilities for enemy
    for ability_id in [AbilityId::Attack] {
        let ability_e = ability_catalog.spawn(ability_id, &mut commands);
        commands.entity(ability_e).insert(Held::<Ability> {
            held_by: enemy,
            _phantom_t: std::marker::PhantomData,
        });
    }

    commands
        .spawn((FightBundle::new(), Name::new("The Fight")))
        .add_children(&[player_character, enemy]);
}

/// Renders the fight selection window positioned below the World Inspector
fn render_fight_selection_window(world: &mut World) {
    let egui_context = world
        .query_filtered::<&mut EguiContext, With<PrimaryEguiContext>>()
        .single(world);

    let Ok(egui_context) = egui_context else {
        return;
    };

    let mut egui_context = egui_context.clone();

    egui::Window::new("Fight Selection")
        .default_size((200.0, 100.0))
        .anchor(egui::Align2::CENTER_TOP, egui::Vec2::new(0.0, 0.0))
        .show(egui_context.get_mut(), |ui| {
            ui.vertical(|ui| {
                if ui.button("Despawn Fight").clicked() {
                    // despawn fight
                    world
                        .run_system_once(despawn_current_fight)
                        .inspect_err(|e| warn!("could not despawn_current_fight: {e:?}"))
                        .ok();
                }

                if ui.button("Basic Fight").clicked() {
                    // despawn first
                    world
                        .run_system_once(despawn_current_fight)
                        .inspect_err(|e| warn!("could not despawn_current_fight: {e:?}"))
                        .ok();

                    // Then spawn new fight
                    world
                        .run_system_once(spawn_basic_fight)
                        .inspect_err(|e| warn!("could spawn_basic_fight: {e:?}"))
                        .ok();
                }
            });
        });
}
