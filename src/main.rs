use abilities::AbilitiesPlugin;
use bevy::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use fight_ui::FightUiPlugin;
use game_logic::{
    faction::Faction, fight::FightBundle, health::Health, AbilityId, AbilitySlot, AbilitySlotType,
    GameLogicPlugin, HasAbilities, HasAbilitySlots,
};

pub mod abilities;
pub mod fight_ui;
pub mod game_logic;
pub mod utils;

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
        .spawn((FightBundle::new(), Name::new("The Fight")))
        .push_children(&[player_character, enemy]);
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
        .add_plugins(DefaultPlugins)
        .add_plugins(WorldInspectorPlugin::new())
        .configure_sets(
            FixedUpdate,
            (
                PerUpdateSet::TimeUpdate,
                PerUpdateSet::LogicUpdate,
                PerUpdateSet::CommandResolution,
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
        .run();
}
