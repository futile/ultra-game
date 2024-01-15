use abilities::AbilitiesPlugin;
use bevy::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use fight_ui::FightUiPlugin;
use game_logic::{
    AbilityId, AbilitySlot, AbilitySlotType, Faction, Fight, GameLogicPlugin, HasAbilities,
    HasAbilitySlots, Health,
};

pub mod abilities;
mod fight_ui;
mod game_logic;

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());

    let player_abilities = commands
        .spawn((Name::new("Player Abilities"),))
        .with_children(|p| {
            p.spawn(AbilityId::Attack);
        })
        .id();

    let player_ability_slots = commands
        .spawn((Name::new("Player Ability Slots"),))
        .with_children(|p| {
            p.spawn(AbilitySlot {
                tpe: AbilitySlotType::WeaponAttack,
            });
            p.spawn(AbilitySlot {
                tpe: AbilitySlotType::ShieldDefend,
            });

            p.spawn(AbilitySlot {
                tpe: AbilitySlotType::WeaponAttack,
            });
            p.spawn(AbilitySlot {
                tpe: AbilitySlotType::ShieldDefend,
            });
            p.spawn(AbilitySlot {
                tpe: AbilitySlotType::WeaponAttack,
            });
            p.spawn(AbilitySlot {
                tpe: AbilitySlotType::ShieldDefend,
            });
        })
        .id();

    let player_character = commands
        .spawn((
            HasAbilitySlots {
                holder: player_ability_slots,
            },
            HasAbilities {
                holder: player_abilities,
            },
            Health::new(100.0),
            Faction::Player,
            Name::new("Player Character"),
        ))
        .id();

    let enemy = commands
        .spawn((Name::new("The Enemy"), Health::new(100.0), Faction::Enemy))
        .id();

    commands
        .spawn((Fight, Name::new("The Fight")))
        .push_children(&[player_character, enemy]);
}

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone, Copy)]
enum PerUpdateSet {
    LogicUpdate,
    CommandSubmission,
    CommandResolution,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(WorldInspectorPlugin::new())
        .configure_sets(
            Update,
            (
                PerUpdateSet::LogicUpdate,
                PerUpdateSet::CommandSubmission,
                PerUpdateSet::CommandResolution,
            )
                .chain(),
        )
        .add_plugins(AbilitiesPlugin)
        .add_plugins(GameLogicPlugin)
        .add_plugins(FightUiPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, bevy::window::close_on_esc)
        .run();
}
