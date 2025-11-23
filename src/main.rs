#![feature(duration_constructors)]

use abilities::AbilitiesPlugin;
use bevy::prelude::*;
use bevy_inspector_egui::{
    bevy_egui::{EguiContext, EguiPlugin, EguiPrimaryContextPass, PrimaryEguiContext, egui},
    bevy_inspector,
};
use big_brain::BigBrainPlugin;
use fight_selection_ui::FightSelectionUiPlugin;
use fight_ui::FightUiPlugin;
use game_logic::GameLogicPlugin;

pub mod abilities;
pub mod fight_selection_ui;
pub mod fight_ui;
pub mod game_logic;
pub mod utils;

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);

    fight_selection_ui::spawn_basic_fight(commands);
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
        .query_filtered::<&mut EguiContext, With<PrimaryEguiContext>>()
        .single(world);

    let Ok(egui_context) = egui_context else {
        return;
    };

    let mut egui_context = egui_context.clone();

    // meh, all of these currently don't work, because _something_ resizes the screen_rect to 512
    // for one frame, which clips this window into that area as well, hiding it behind the fight
    // window.
    // let top_right = dbg!(egui_context.get_mut().screen_rect().right_top());
    // .pivot(egui::Align2::RIGHT_TOP)
    // .default_pos(dbg!(top_right - egui::Vec2::new(320.0, 0.0)))
    // .default_pos(egui::Pos2::new(960.0, 0.0))

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
        .add_plugins(EguiPlugin::default())
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
        .add_plugins(FightSelectionUiPlugin)
        .add_plugins(FightUiPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, close_on_esc)
        .add_systems(EguiPrimaryContextPass, positioned_world_inspector)
        .run();
}
