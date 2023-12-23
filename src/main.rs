use ability_catalog::AbilityCatalogPlugin;
use bevy::prelude::*;
use bevy_inspector_egui::{
    bevy_egui::{egui, EguiContexts},
    egui::Visuals,
    quick::WorldInspectorPlugin,
};
use core_logic::{
    AbilityId, AbilitySlot, AbilitySlotType, AbilitySlots, CoreLogicPlugin, Enemy, Fight,
    HasAbilities, PlayerCharacter,
};
use fight_board_plugin::FightBoardPlugin;
use smallvec::smallvec;

mod ability_catalog;
mod core_logic;
mod fight_board_plugin;

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());

    let player_abilities = commands
        .spawn((Name::new("Player Abilities"),))
        .with_children(|p| {
            p.spawn(AbilityId::Attack);
        })
        .id();

    let player_character = commands
        .spawn((
            PlayerCharacter,
            AbilitySlots(smallvec![
                AbilitySlot {
                    tpe: AbilitySlotType::WeaponAttack
                },
                AbilitySlot {
                    tpe: AbilitySlotType::ShieldDefend
                }
            ]),
            HasAbilities {
                holder: player_abilities,
            },
            Name::new("Player Character"),
        ))
        .id();

    let enemy = commands.spawn((Enemy, Name::new("The Enemy"))).id();

    commands.spawn((
        Fight {
            player_character,
            enemy,
        },
        Name::new("The Fight"),
    ));
}

fn ui_example_system(mut contexts: EguiContexts) {
    // enable light style for primary window: https://github.com/emilk/egui/discussions/1627
    contexts
        .ctx_mut()
        .style_mut(|style| style.visuals = Visuals::light());

    egui::Window::new("Hello").show(contexts.ctx_mut(), |ui| {
        ui.label("world");
        ui.label("foo");
    });
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(WorldInspectorPlugin::new())
        .add_plugins(AbilityCatalogPlugin)
        .add_plugins(CoreLogicPlugin)
        .add_plugins(FightBoardPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, bevy::window::close_on_esc)
        .add_systems(Update, ui_example_system)
        .run();
}
