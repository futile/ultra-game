use ability_catalog::AbilityCatalogPlugin;
use bevy::prelude::*;
use bevy_inspector_egui::{
    bevy_egui::{egui, EguiContexts},
    egui::{Id, RichText, Ui, Visuals},
    quick::WorldInspectorPlugin,
};
use core_logic::{
    AbilityId, AbilitySlot, AbilitySlotType, AbilitySlots, CoreLogicPlugin, Fight, HasAbilities,
};
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

    let enemy = commands.spawn(Name::new("The Enemy")).id();

    commands.spawn((
        Fight {
            player_character,
            enemy,
        },
        Name::new("The Fight"),
    ));
}

fn ui_ability_slots(ui: &mut Ui, slots: &AbilitySlots) {
    // TODO: add colors (again) at some point (if it fits..)
    // old colors for reference:
    // AbilitySlotType::WeaponAttack => Color::LIME_GREEN,
    // AbilitySlotType::ShieldDefend => Color::PINK,

    ui.heading("Ability Slots");
    ui.indent(ui.id().with("ability_slots"), |ui: &mut Ui| {
        for slot in slots.0.iter() {
            ui.label(match slot.tpe {
                AbilitySlotType::WeaponAttack => "Weapon Attack",
                AbilitySlotType::ShieldDefend => "Shield Defend",
            });
        }
    });
}

fn ui_fight_column(
    ui: &mut Ui,
    e: Entity,
    names: &Query<&Name>,
    ability_slots: &Query<&AbilitySlots>,
) {
    ui.indent(ui.id().with("entity_name"), |ui: &mut Ui| {
        if let Some(name) = names.get(e).ok() {
            ui.label(name.as_str());
        } else {
            ui.label("<No Name>");
        }
    });

    if let Some(slots) = ability_slots.get(e).ok() {
        ui.add_space(10.);
        ui_ability_slots(ui, slots);
    }
}

fn ui_example_system(
    _commands: Commands,
    fights: Query<(Entity, &Fight)>,
    names: Query<&Name>,
    slots: Query<&AbilitySlots>,
    mut contexts: EguiContexts,
) {
    // context for the primary (so far, only) window
    let ui_ctx = contexts.ctx_mut();

    // enable light style: https://github.com/emilk/egui/discussions/1627
    ui_ctx.style_mut(|style| style.visuals = Visuals::light());

    for (e, fight) in fights.iter() {
        egui::Window::new("Fight")
            .id(Id::new(e))
            .show(ui_ctx, |ui: &mut Ui| {
                ui.columns(2, |columns: &mut [Ui]| {
                    columns[0].label(RichText::new("Player").heading().strong());
                    ui_fight_column(&mut columns[0], fight.player_character, &names, &slots);

                    columns[1].label(RichText::new("Enemy").heading().strong());
                    ui_fight_column(&mut columns[1], fight.enemy, &names, &slots);
                });
            });
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(WorldInspectorPlugin::new())
        .add_plugins(AbilityCatalogPlugin)
        .add_plugins(CoreLogicPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, bevy::window::close_on_esc)
        .add_systems(Update, ui_example_system)
        .run();
}
