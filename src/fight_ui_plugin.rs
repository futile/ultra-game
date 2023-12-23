use bevy::prelude::*;
use bevy_inspector_egui::{
    bevy_egui::EguiContexts,
    egui::{self, Id, RichText, Ui, Visuals},
};

use crate::{AbilitySlotType, AbilitySlots, Fight, HasAbilities};

pub struct FightUiPlugin;

impl Plugin for FightUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, ui_fight_windows);
    }
}

fn ui_fight_windows(
    _commands: Commands,
    fights: Query<(Entity, &Fight)>,
    names: Query<&Name>,
    ability_slots: Query<&AbilitySlots>,
    has_abilities: Query<&HasAbilities>,
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
                    ui_fight_column(
                        &mut columns[0],
                        fight.player_character,
                        &names,
                        &ability_slots,
                        &has_abilities,
                    );

                    columns[1].label(RichText::new("Enemy").heading().strong());
                    ui_fight_column(
                        &mut columns[1],
                        fight.enemy,
                        &names,
                        &ability_slots,
                        &has_abilities,
                    );
                });
            });
    }
}

fn ui_fight_column(
    ui: &mut Ui,
    e: Entity,
    names: &Query<&Name>,
    ability_slots: &Query<&AbilitySlots>,
    has_abilities: &Query<&HasAbilities>,
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

    if let Some(abilities) = has_abilities.get(e).ok() {
        ui.add_space(10.);
        ui_abilities(ui, abilities);
    }
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

fn ui_abilities(ui: &mut Ui, _abilities: &HasAbilities) {
    ui.heading("Abilities");

    ui.indent(ui.id().with("abilities"), |ui: &mut Ui| {
        ui.label("TODO: render abilities here");
    });
}
