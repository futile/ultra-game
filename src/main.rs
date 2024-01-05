use abilities::AbilitiesPlugin;
use bevy::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use core_logic::{
    AbilityId, AbilitySlot, AbilitySlotType, CoreLogicPlugin, Fight, HasAbilities, HasAbilitySlots,
};
use fight_ui::FightUiPlugin;

pub mod abilities;
mod core_logic;
mod fight_ui;

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
            Name::new("Player Character"),
        ))
        .id();

    let enemy = commands.spawn(Name::new("The Enemy")).id();

    commands.spawn((
        Fight {
            player_character,
            enemy,
        },
        Name::new("The Fight"),
    ));
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
        .add_plugins(CoreLogicPlugin)
        .add_plugins(FightUiPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, bevy::window::close_on_esc)
        .run();
}
