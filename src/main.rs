#![feature(duration_constructors)]

use std::time::Duration;

use abilities::AbilitiesPlugin;
use bevy::{prelude::*, window::PrimaryWindow};
use bevy_inspector_egui::{
    bevy_egui::{EguiContext, EguiContextPass, EguiPlugin, egui},
    bevy_inspector,
};
use big_brain::{BigBrainPlugin, prelude::*};
use fight_ui::FightUiPlugin;
use game_logic::{
    GameLogicPlugin,
    ability::AbilityId,
    ability_slots::{AbilitySlot, AbilitySlotType},
    ai_behavior::{AttackPlayerAction, CanAttackPlayerScorer},
    faction::Faction,
    fight::FightBundle,
    health::Health,
};

use crate::utils::holds_held::Held;

pub mod abilities;
pub mod fight_ui;
pub mod game_logic;
pub mod utils;

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);

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

// from https://github.com/bevyengine/bevy/pull/12859
fn close_on_esc(
    mut commands: Commands,
    focused_windows: Query<(Entity, &Window)>,
    input: Res<ButtonInput<KeyCode>>,
) {
    for (window, focus) in focused_windows.iter() {
        if !focus.focused {
            continue;
        }

        if input.just_pressed(KeyCode::Escape) {
            commands.entity(window).despawn();
        }
    }
}

/// Custom world inspector system positioned in top-right corner, so it doesn't overlap with the
/// fight window we open.
///
/// See https://github.com/jakobhellermann/bevy-inspector-egui/blob/v0.31.0/crates/bevy-inspector-egui/src/quick.rs#L92-L110
/// for where the code was derived from.
fn positioned_world_inspector(world: &mut World) {
    let egui_context = world
        .query_filtered::<&mut EguiContext, With<PrimaryWindow>>()
        .single(world);

    let Ok(egui_context) = egui_context else {
        return;
    };

    let mut egui_context = egui_context.clone();

    egui::Window::new("World Inspector")
        .default_size((320.0, 160.0))
        .anchor(egui::Align2::RIGHT_TOP, egui::Vec2::ZERO)
        .show(egui_context.get_mut(), |ui| {
            egui::ScrollArea::both().show(ui, |ui| {
                bevy_inspector::ui_for_world(world, ui);
                ui.allocate_space(ui.available_size());
            });
        });
}

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone, Copy)]
enum PerUpdateSet {
    // FixedUpdate
    TimeUpdate,
    LogicUpdate,
    CommandSubmission,
    DamageResolution,
    FightEndChecking,

    // Update
    CommandResolution,
}

fn main() {
    App::new()
        // this... somehow warns for weird ambiguities between systems where one is in
        // FixedUpdate, and the other is in Update.. no idea why. so turning off for now,
        // but will need some kind of solution for ablity-cast-ordering in CommandResolution.
        // .edit_schedule(FixedUpdate, |schedule| {
        //     schedule.set_build_settings(ScheduleBuildSettings {
        //         ambiguity_detection: LogLevel::Warn,
        //         ..default()
        //     });
        // })
        // .edit_schedule(Update, |schedule| {
        //     schedule.set_build_settings(ScheduleBuildSettings {
        //         ambiguity_detection: LogLevel::Warn,
        //         ..default()
        //     });
        // })
        .add_plugins(DefaultPlugins)
        .add_plugins(EguiPlugin {
            enable_multipass_for_primary_context: true,
        })
        .add_plugins(BigBrainPlugin::new(PreUpdate))
        .configure_sets(
            FixedUpdate,
            (
                PerUpdateSet::CommandResolution,
                PerUpdateSet::TimeUpdate,
                PerUpdateSet::LogicUpdate,
                PerUpdateSet::DamageResolution,
                PerUpdateSet::FightEndChecking,
            )
                .chain(),
        )
        .configure_sets(Update, (PerUpdateSet::CommandSubmission,).chain())
        .add_plugins(AbilitiesPlugin)
        .add_plugins(GameLogicPlugin)
        .add_plugins(FightUiPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, close_on_esc)
        .add_systems(EguiContextPass, positioned_world_inspector)
        .run();
}
