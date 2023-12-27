use bevy::prelude::*;
use bevy_inspector_egui::{
    bevy_egui::EguiContexts,
    egui::{self, Id, Key, KeyboardShortcut, Modifiers, RichText, Ui, Visuals},
};

use super::{ui_utils, FightWindow};
use crate::{
    core_logic::{AbilityId, AbilitySlot},
    AbilitySlotType, Fight, HasAbilities, HasAbilitySlots,
};

#[derive(Debug, Default, Component, Reflect)]
pub struct FightWindowUiState {
    player_column_state: FightColumnUiState,
    enemy_column_state: FightColumnUiState,
}

pub fn render_fight_windows(
    mut _commands: Commands,
    mut fight_windows: Query<(Entity, &mut FightWindow)>,
    fights: Query<&Fight>,
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

    for (window_e, fight_window) in &mut fight_windows {
        let fight = fights
            .get(fight_window.model)
            .expect("FightWindow.model doesn't have a Fight");

        let mut ui_state = fight_window.map_unchanged(|fw| &mut fw.ui_state);

        egui::Window::new("Fight")
            .id(Id::new(window_e))
            .show(ui_ctx, |ui: &mut Ui| {
                ui.columns(2, |columns: &mut [Ui]| {
                    columns[0].label(RichText::new("Player").heading().strong());

                    ui_fight_column(
                        &mut columns[0],
                        &mut ui_state.player_column_state,
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
                        &mut ui_state.enemy_column_state,
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

#[derive(Debug, Default, Reflect)]
struct FightColumnUiState {
    abilities_section_state: AbilitySlotsSectionUiState,
}

#[derive(Debug, Default, Reflect)]
struct AbilitySlotsSectionUiState {
    selected_slot: Option<Entity>,
}

fn ui_fight_column(
    ui: &mut Ui,
    ui_column_state: &mut FightColumnUiState,
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
        ui_ability_slots(
            ui,
            &mut ui_column_state.abilities_section_state,
            slots,
            children,
            ability_slots,
        );
    }

    if let Some(abilities) = has_abilities.get(e).ok() {
        ui.add_space(10.);
        ui_abilities(ui, abilities, children, ability_ids);
    }
}

fn ui_ability_slots(
    ui: &mut Ui,
    abilities_section_state: &mut AbilitySlotsSectionUiState,
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
        for (idx, child) in children
            .get(slots.holder)
            .expect("HasAbilitySlots.holder without Children")
            .iter()
            .enumerate()
        {
            let slot = ability_slots
                .get(*child)
                .expect("ability slot without AbilitySlot");

            let keyboard_shortcut: Option<KeyboardShortcut> = {
                let key: Option<Key> = match idx {
                    0 => Some(Key::Num1),
                    1 => Some(Key::Num2),
                    2 => Some(Key::Num3),
                    3 => Some(Key::Num4),
                    _ => None,
                };

                key.map(|key| KeyboardShortcut::new(Modifiers::NONE, key))
            };

            if let Some(shortcut) = keyboard_shortcut {
                ui.input_mut(|i| {
                    if i.consume_shortcut(&shortcut) {
                        abilities_section_state.selected_slot =
                            match abilities_section_state.selected_slot {
                                // if selected before, toggle selection off
                                Some(selected) if selected == *child => None,
                                // otherwise select this slot
                                _ => Some(*child),
                            };
                    }
                });
            }

            ui.horizontal(|ui: &mut Ui| {
                let leading_text: String = match keyboard_shortcut {
                    Some(shortcut) => ui.ctx().format_shortcut(&shortcut),
                    None => String::from(" "),
                };

                ui.monospace(leading_text);

                ui_utils::un_selectable_value(
                    ui,
                    &mut abilities_section_state.selected_slot,
                    *child,
                    match slot.tpe {
                        AbilitySlotType::WeaponAttack => "Weapon Attack",
                        AbilitySlotType::ShieldDefend => "Shield Defend",
                    },
                );
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
            .expect("HasAbilities.holder without Children")
        {
            let ability_id = ability_ids.get(*child).expect("ability without AbilityId");
            ui.label(format!("{:?}", ability_id));
        }
    });
}
