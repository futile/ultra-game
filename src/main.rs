use ability_catalog::AbilityCatalogPlugin;
use bevy::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
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

    let player_character = commands
        .spawn((
            PlayerCharacter,
            AbilitySlots(smallvec![AbilitySlot {
                tpe: AbilitySlotType::WeaponAttack
            }]),
            HasAbilities(smallvec![AbilityId::Attack]),
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

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(WorldInspectorPlugin::new())
        .add_plugins(AbilityCatalogPlugin)
        .add_plugins(CoreLogicPlugin)
        .add_plugins(FightBoardPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, bevy::window::close_on_esc)
        .run();
}
