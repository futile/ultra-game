use bevy::prelude::*;
use bevy_inspector_egui::{
    bevy_egui::EguiContexts,
    egui::{self, Id, RichText, Ui, Visuals},
};

use crate::{
    core_logic::{AbilityId, AbilitySlot},
    AbilitySlotType, Fight, HasAbilities, HasAbilitySlots,
};

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
    has_ability_slots: Query<&HasAbilitySlots>,
    has_abilities: Query<&HasAbilities>,
    children: Query<&Children>,
    ability_ids: Query<&AbilityId>,
    ability_slots: Query<&AbilitySlot>,
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
                        &has_ability_slots,
                        &has_abilities,
                        &children,
                        &ability_ids,
                        &ability_slots,
                    );

                    columns[1].label(RichText::new("Enemy").heading().strong());
                    ui_fight_column(
                        &mut columns[1],
                        fight.enemy,
                        &names,
                        &has_ability_slots,
                        &has_abilities,
                        &children,
                        &ability_ids,
                        &ability_slots,
                    );
                });
            });
    }
}

fn ui_fight_column(
    ui: &mut Ui,
    e: Entity,
    names: &Query<&Name>,
    has_ability_slots: &Query<&HasAbilitySlots>,
    has_abilities: &Query<&HasAbilities>,
    children: &Query<&Children>,
    ability_ids: &Query<&AbilityId>,
    ability_slots: &Query<&AbilitySlot>,
) {
    ui.indent(ui.id().with("entity_name"), |ui: &mut Ui| {
        if let Some(name) = names.get(e).ok() {
            ui.label(name.as_str());
        } else {
            ui.label("<No Name>");
        }
    });

    if let Some(slots) = has_ability_slots.get(e).ok() {
        ui.add_space(10.);
        ui_ability_slots(ui, slots, children, ability_slots);
    }

    if let Some(abilities) = has_abilities.get(e).ok() {
        ui.add_space(10.);
        ui_abilities(ui, abilities, children, ability_ids);
    }
}

fn ui_ability_slots(
    ui: &mut Ui,
    slots: &HasAbilitySlots,
    children: &Query<&Children>,
    ability_slots: &Query<&AbilitySlot>,
) {
    // TODO: add colors (again) at some point (if it fits..)
    // old colors for reference:
    // AbilitySlotType::WeaponAttack => Color::LIME_GREEN,
    // AbilitySlotType::ShieldDefend => Color::PINK,

    ui.heading("Ability Slots");

    ui.indent(ui.id().with("ability_slots"), |ui: &mut Ui| {
        for child in children
            .get(slots.holder)
            .expect("HasAbilitySlots.holder without children")
        {
            let slot = ability_slots
                .get(*child)
                .expect("ability slot without AbilitySlotType");

            ui.label(match slot.tpe {
                AbilitySlotType::WeaponAttack => "Weapon Attack",
                AbilitySlotType::ShieldDefend => "Shield Defend",
            });
        }
    });
}

fn ui_abilities(
    ui: &mut Ui,
    abilities: &HasAbilities,
    children: &Query<&Children>,
    ability_ids: &Query<&AbilityId>,
) {
    ui.heading("Abilities");

    ui.indent(ui.id().with("abilities"), |ui: &mut Ui| {
        for child in children
            .get(abilities.holder)
            .expect("HasAbilities.holder without children")
        {
            let ability_id = ability_ids.get(*child).expect("ability without AbilityId");
            ui.label(format!("{:?}", ability_id));
        }
    });
}
