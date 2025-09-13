use std::time::Duration;

use bevy::{prelude::*, window::PrimaryWindow};
use bevy_inspector_egui::bevy_egui::{EguiContext, EguiContextPass, egui};
use big_brain::prelude::*;

use crate::{
    game_logic::{
        ability::AbilityId,
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
        app.add_systems(EguiContextPass, render_fight_selection_window);
    }
}

/// Despawns the current fight by finding the Fight entity and recursively despawning
/// its children (player and enemy) and the fight itself.
pub fn despawn_current_fight(commands: &mut Commands, fights: &Query<Entity, With<Fight>>) {
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
pub fn spawn_basic_fight(commands: &mut Commands) {
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
        .with_related_entities::<Held<AbilityId>>(|commands| {
            commands.spawn(AbilityId::Attack);
            commands.spawn(AbilityId::NeedlingHex);
            commands.spawn(AbilityId::ChargedStrike);
        })
        .id();

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
        .with_related_entities::<Held<AbilityId>>(|commands| {
            commands.spawn(AbilityId::Attack);
        })
        .id();

    commands
        .spawn((FightBundle::new(), Name::new("The Fight")))
        .add_children(&[player_character, enemy]);
}

/// Renders the fight selection window positioned below the World Inspector
fn render_fight_selection_window(world: &mut World) {
    let egui_context = world
        .query_filtered::<&mut EguiContext, With<PrimaryWindow>>()
        .single(world);

    let Ok(egui_context) = egui_context else {
        return;
    };

    let mut egui_context = egui_context.clone();

    egui::Window::new("Fight Selection")
        .default_size((200.0, 100.0))
        .anchor(egui::Align2::RIGHT_TOP, egui::Vec2::new(0.0, 180.0))
        .show(egui_context.get_mut(), |ui| {
            ui.vertical(|ui| {
                if ui.button("Despawn Fight").clicked() {
                    // First collect all the data we need
                    let fights: Vec<Entity> = world
                        .query_filtered::<Entity, With<Fight>>()
                        .iter(world)
                        .collect();
                    let mut fights_with_children: Vec<(Entity, Vec<Entity>)> = Vec::new();

                    for fight_e in fights {
                        let children = if let Some(fight_children) = world.get::<Children>(fight_e)
                        {
                            fight_children.to_vec()
                        } else {
                            Vec::new()
                        };
                        fights_with_children.push((fight_e, children));
                    }

                    // Now get commands and do the despawning
                    let mut commands = world.commands();

                    for (fight_e, children) in fights_with_children {
                        // Despawn all children (player and enemy)
                        for child_e in children {
                            commands.entity(child_e).despawn();
                        }

                        // Despawn the fight entity itself
                        commands.entity(fight_e).despawn();
                    }
                }

                if ui.button("Basic Fight").clicked() {
                    // First collect all the data we need for despawning
                    let fights: Vec<Entity> = world
                        .query_filtered::<Entity, With<Fight>>()
                        .iter(world)
                        .collect();
                    let mut fights_with_children: Vec<(Entity, Vec<Entity>)> = Vec::new();

                    for fight_e in fights {
                        let children = if let Some(fight_children) = world.get::<Children>(fight_e)
                        {
                            fight_children.to_vec()
                        } else {
                            Vec::new()
                        };
                        fights_with_children.push((fight_e, children));
                    }

                    // Now get commands and do the despawning and spawning
                    let mut commands = world.commands();

                    for (fight_e, children) in fights_with_children {
                        // Despawn all children (player and enemy)
                        for child_e in children {
                            commands.entity(child_e).despawn();
                        }

                        // Despawn the fight entity itself
                        commands.entity(fight_e).despawn();
                    }

                    // Then spawn new fight
                    spawn_basic_fight(&mut commands);
                }
            });
        });
}
