use ability_catalog::AbilityCatalogPlugin;
use bevy::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use core_logic::{
    AbilitySlot, AbilitySlotType, Character, CoreLogicPlugin, Enemy, Fight, PlayerCharacter,
};
use fight_board_plugin::FightBoardPlugin;

mod ability_catalog;
mod core_logic;
mod fight_board_plugin;

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());

    let the_player: Character = Character {
        slots: smallvec::smallvec![AbilitySlot {
            tpe: AbilitySlotType::WeaponAttack
        }],
    };

    let player_character = commands
        .spawn((
            PlayerCharacter {
                character: the_player,
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
