use bevy::prelude::*;
use bevy_inspector_egui::{
    bevy_egui::EguiContexts,
    egui::{self, Id, Key, KeyboardShortcut, Modifiers, RichText, Ui, Visuals},
};

use super::FightWindow;
use crate::{
    ability_catalog::ability_catalog,
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
        ui_abilities(
            ui,
            abilities,
            children,
            ability_ids,
            ability_slots,
            ui_column_state,
        )
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
        for (idx, &slot_e) in children
            .get(slots.holder)
            .expect("HasAbilitySlots.holder without Children")
            .iter()
            .enumerate()
        {
            let slot = ability_slots
                .get(slot_e)
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

            let shortcut_pressed: bool = match keyboard_shortcut {
                // TODO: somehow check if the current egui window has keyboard focus.
                // Window.show().response.has_focus() is always false, so dunno how to do this.
                Some(shortcut) => ui.input_mut(|i| i.consume_shortcut(&shortcut)),
                None => false,
            };

            let shortcut_text: String = match keyboard_shortcut {
                Some(shortcut) => ui.ctx().format_shortcut(&shortcut),
                None => String::from(" "),
            };

            let slot_is_selected: bool = abilities_section_state
                .selected_slot
                .is_some_and(|s| s == slot_e);

            ui.horizontal(|ui: &mut Ui| {
                ui.monospace(shortcut_text);

                let mut label_response = ui.selectable_label(
                    slot_is_selected,
                    match slot.tpe {
                        AbilitySlotType::WeaponAttack => "Weapon Attack",
                        AbilitySlotType::ShieldDefend => "Shield Defend",
                    },
                );

                if shortcut_pressed || label_response.clicked() {
                    abilities_section_state.selected_slot =
                        if slot_is_selected { None } else { Some(slot_e) };

                    // not 100% sure why this is needed, but `Ui::selectable_value()` does it as
                    // well, so it might be necessary.
                    label_response.mark_changed();
                }
            });
        }
    });
}

fn ui_abilities(
    ui: &mut Ui,
    abilities: &HasAbilities,
    children: &Query<&Children>,
    ability_ids: &Query<&AbilityId>,
    ability_slots: &Query<&AbilitySlot>,
    ui_state: &mut FightColumnUiState,
) {
    let selected_slot = ui_state
        .abilities_section_state
        .selected_slot
        .and_then(|s| ability_slots.get(s).ok());

    ui.heading("Abilities");

    ui.indent(ui.id().with("abilities"), |ui: &mut Ui| {
        for (idx, ability_id_e) in children
            .get(abilities.holder)
            .expect("HasAbilities.holder without Children")
            .iter()
            .enumerate()
        {
            let ability_id = ability_ids
                .get(*ability_id_e)
                .expect("ability without AbilityId");
            let ability = ability_catalog()
                .get(ability_id)
                .expect(&format!("AbilityId `{:?}` not in catalog", ability_id));
            let ability_usable = ability.can_use(selected_slot);

            let keyboard_shortcut: Option<KeyboardShortcut> = {
                let key: Option<Key> = match idx {
                    0 => Some(Key::X),
                    // 1 => Some(Key::Num2),
                    _ => None,
                };

                key.map(|key| KeyboardShortcut::new(Modifiers::NONE, key))
            };

            let shortcut_pressed: bool = match keyboard_shortcut {
                // TODO: somehow check if the current egui window has keyboard focus.
                // Window.show().response.has_focus() is always false, so dunno how to do this.
                Some(shortcut) => ui.input_mut(|i| i.consume_shortcut(&shortcut)),
                None => false,
            };

            let shortcut_text: String = match keyboard_shortcut {
                Some(shortcut) => ui.ctx().format_shortcut(&shortcut),
                None => String::from(" "),
            };

            ui.add_enabled_ui(ability_usable, |ui: &mut Ui| {
                ui.horizontal(|ui: &mut Ui| {
                    ui.monospace(shortcut_text);

                    let ability_button = ui.add(egui::Button::new(format!("{}", ability.name)));

                    if shortcut_pressed || ability_button.clicked() {
                        // TODO: actual logic for abilitiess somehow.
                        // Fire an event, which might be a `Command` for the entity,
                        // or the cast itself, so it can be resolved?
                        // Think about structure.
                        println!("Ability {:?} used", ability_id);

                        // clear the selected slot, because it was used.
                        ui_state.abilities_section_state.selected_slot = None;
                    }
                });
            });
        }
    });
}
