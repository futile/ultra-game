use bevy::prelude::*;
use bevy_inspector_egui::{
    bevy_egui::EguiContexts,
    egui::{self, Id, Key, KeyboardShortcut, Modifiers, RichText, Ui, Visuals},
};
use itertools::Itertools;

use super::FightWindow;
use crate::{
    abilities::AbilityCatalog,
    game_logic::{commands, AbilityId, AbilitySlot, Faction, Health},
    AbilitySlotType, Fight, HasAbilities, HasAbilitySlots,
};

#[derive(Debug, Component, Reflect)]
pub struct FightWindowUiState {
    player_column_state: FightColumnUiState,
    enemy_column_state: FightColumnUiState,
}

impl Default for FightWindowUiState {
    fn default() -> Self {
        Self {
            player_column_state: FightColumnUiState::new(true),
            enemy_column_state: FightColumnUiState::new(false),
        }
    }
}

pub fn render_fight_windows(
    mut _commands: Commands,
    mut fight_windows: Query<(Entity, &mut FightWindow)>,
    fights: Query<&Fight>,
    factions: Query<(Entity, &Faction)>,
    names: Query<&Name>,
    healths: Query<&Health>,
    has_ability_slots: Query<&HasAbilitySlots>,
    has_abilities: Query<&HasAbilities>,
    children: Query<&Children>,
    ability_ids: Query<&AbilityId>,
    ability_slots: Query<&AbilitySlot>,
    ability_catalog: Res<AbilityCatalog>,
    mut cast_ability: EventWriter<commands::CastAbility>,
    mut contexts: EguiContexts,
) {
    // context for the primary (so far, only) window
    let ui_ctx = contexts.ctx_mut();

    // enable light style: https://github.com/emilk/egui/discussions/1627
    ui_ctx.style_mut(|style| style.visuals = Visuals::light());

    for (window_e, fight_window) in &mut fight_windows {
        let fight_e = fight_window.model;

        fights
            .get(fight_e)
            .expect("FightWindow.model doesn't have a Fight");

        let fight_children = children.get(fight_e).expect("Fight without Children");
        let player_entity = factions
            .iter_many(fight_children)
            .filter(|(_e, faction)| **faction == Faction::Player)
            .at_most_one()
            .ok()
            .flatten()
            .unwrap()
            .0;

        let enemy_entity = factions
            .iter_many(fight_children)
            .filter(|(_e, faction)| **faction == Faction::Enemy)
            .at_most_one()
            .ok()
            .flatten()
            .unwrap()
            .0;

        let mut ui_state = fight_window.map_unchanged(|fw| &mut fw.ui_state);

        egui::Window::new("Fight")
            .id(Id::new(window_e))
            .show(ui_ctx, |ui: &mut Ui| {
                ui.columns(2, |columns: &mut [Ui]| {
                    columns[0].label(RichText::new("Player").heading().strong());

                    ui_fight_column(
                        &mut columns[0],
                        &mut ui_state.player_column_state,
                        player_entity,
                        fight_e,
                        &names,
                        &healths,
                        &has_ability_slots,
                        &has_abilities,
                        &children,
                        &ability_ids,
                        &ability_slots,
                        &ability_catalog,
                        &mut cast_ability,
                    );

                    columns[1].label(RichText::new("Enemy").heading().strong());

                    ui_fight_column(
                        &mut columns[1],
                        &mut ui_state.enemy_column_state,
                        enemy_entity,
                        fight_e,
                        &names,
                        &healths,
                        &has_ability_slots,
                        &has_abilities,
                        &children,
                        &ability_ids,
                        &ability_slots,
                        &ability_catalog,
                        &mut cast_ability,
                    );
                });
            });
    }
}

#[derive(Debug, Reflect)]
struct FightColumnUiState {
    abilities_section_state: AbilitySlotsSectionUiState,
    user_interactable: bool,
}

impl FightColumnUiState {
    fn new(user_interactable: bool) -> FightColumnUiState {
        FightColumnUiState {
            abilities_section_state: AbilitySlotsSectionUiState::new(user_interactable),
            user_interactable,
        }
    }
}

#[derive(Debug, Reflect)]
struct AbilitySlotsSectionUiState {
    selected_slot: Option<Entity>,
    user_interactable: bool,
}

impl AbilitySlotsSectionUiState {
    fn new(user_interactable: bool) -> AbilitySlotsSectionUiState {
        AbilitySlotsSectionUiState {
            selected_slot: None,
            user_interactable,
        }
    }
}

fn ui_fight_column(
    ui: &mut Ui,
    ui_column_state: &mut FightColumnUiState,
    model_e: Entity,
    fight_e: Entity,
    names: &Query<&Name>,
    healths: &Query<&Health>,
    has_ability_slots: &Query<&HasAbilitySlots>,
    has_abilities: &Query<&HasAbilities>,
    children: &Query<&Children>,
    ability_ids: &Query<&AbilityId>,
    ability_slots: &Query<&AbilitySlot>,
    ability_catalog: &Res<AbilityCatalog>,
    cast_ability: &mut EventWriter<commands::CastAbility>,
) {
    ui.indent(ui.id().with("entity_overview_section"), |ui: &mut Ui| {
        if let Some(name) = names.get(model_e).ok() {
            ui.label(name.as_str());
        } else {
            ui.label("<No Name>");
        }

        if let Some(health) = healths.get(model_e).ok() {
            ui.label(format!("Health: {}/{}", health.current, health.max));
        } else {
            ui.label("<No Health>");
        }
    });

    if let Some(slots) = has_ability_slots.get(model_e).ok() {
        ui.add_space(10.);
        ui_ability_slots(
            ui,
            &mut ui_column_state.abilities_section_state,
            slots,
            children,
            ability_slots,
        );
    }

    if let Some(abilities) = has_abilities.get(model_e).ok() {
        ui.add_space(10.);
        ui_abilities(
            ui,
            model_e,
            fight_e,
            abilities,
            children,
            ability_ids,
            ability_slots,
            ability_catalog,
            cast_ability,
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

    let user_interactable = abilities_section_state.user_interactable;

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

            let keyboard_shortcut: Option<KeyboardShortcut> = if user_interactable {
                let key: Option<Key> = match idx {
                    0 => Some(Key::Num1),
                    1 => Some(Key::Num2),
                    2 => Some(Key::Num3),
                    3 => Some(Key::Num4),
                    _ => None,
                };

                key.map(|key| KeyboardShortcut::new(Modifiers::NONE, key))
            } else {
                None
            };

            let slot_is_selected: bool = abilities_section_state
                .selected_slot
                .is_some_and(|s| s == slot_e);

            ui.horizontal(|ui: &mut Ui| {
                let shortcut_pressed = monospace_checked_shortcut(ui, keyboard_shortcut.as_ref());

                let mut label_response = ui
                    .add_enabled_ui(user_interactable, |ui: &mut Ui| {
                        ui.selectable_label(
                            slot_is_selected,
                            match slot.tpe {
                                AbilitySlotType::WeaponAttack => "Weapon Attack",
                                AbilitySlotType::ShieldDefend => "Shield Defend",
                            },
                        )
                    })
                    .inner;

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
    model: Entity,
    fight_e: Entity,
    abilities: &HasAbilities,
    children: &Query<&Children>,
    ability_ids: &Query<&AbilityId>,
    ability_slots: &Query<&AbilitySlot>,
    ability_catalog: &Res<AbilityCatalog>,
    cast_ability: &mut EventWriter<commands::CastAbility>,
    ui_state: &mut FightColumnUiState,
) {
    let user_interactable = ui_state.user_interactable;
    let selected_slot_e = ui_state.abilities_section_state.selected_slot;
    let selected_slot = selected_slot_e.and_then(|s| ability_slots.get(s).ok());

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
            let ability = ability_catalog
                .0
                .get(ability_id)
                .expect(&format!("AbilityId `{:?}` not in catalog", ability_id));
            let ability_usable = ability.can_use(selected_slot);

            let keyboard_shortcut: Option<KeyboardShortcut> = if user_interactable {
                let key: Option<Key> = match idx {
                    0 => Some(Key::X),
                    // 1 => Some(Key::Num2),
                    _ => None,
                };

                key.map(|key| KeyboardShortcut::new(Modifiers::NONE, key))
            } else {
                None
            };

            ui.add_enabled_ui(ability_usable, |ui: &mut Ui| {
                ui.horizontal(|ui: &mut Ui| {
                    let shortcut_pressed =
                        monospace_checked_shortcut(ui, keyboard_shortcut.as_ref());

                    let ability_button = ui.add_enabled(
                        user_interactable,
                        egui::Button::new(format!("{}", ability.name)),
                    );

                    if ability_usable && (shortcut_pressed || ability_button.clicked()) {
                        cast_ability.send(commands::CastAbility {
                            caster_e: model,
                            slot_e: selected_slot_e,
                            ability_e: *ability_id_e,
                            fight_e,
                        });

                        // clear the selected slot, because it was used.
                        ui_state.abilities_section_state.selected_slot = None;
                    }
                });
            });
        }
    });
}

fn monospace_checked_shortcut(ui: &mut Ui, shortcut: Option<&KeyboardShortcut>) -> bool {
    let shortcut_pressed: bool = match shortcut {
        // TODO: somehow check if the current egui window has keyboard focus.
        // Window.show().response.has_focus() is always false, so dunno how to do this.
        Some(shortcut) => ui.input_mut(|i| i.consume_shortcut(&shortcut)),
        None => false,
    };

    let shortcut_text: String = match shortcut {
        Some(shortcut) => ui.ctx().format_shortcut(&shortcut),
        None => String::from(" "),
    };

    ui.monospace(shortcut_text);

    shortcut_pressed
}
