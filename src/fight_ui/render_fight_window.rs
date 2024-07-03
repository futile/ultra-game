use bevy::{ecs::system::SystemState, prelude::*};
use bevy_inspector_egui::{
    bevy_egui::EguiContexts,
    egui::{self, Id, Key, KeyboardShortcut, Modifiers, RichText, Ui, Visuals},
};
use itertools::Itertools;

use super::FightWindow;
use crate::{
    abilities::AbilityInterface,
    game_logic::{commands, faction::Faction, fight::FightResult, health::Health, AbilitySlot},
    utils::egui_systems::run_ui_system,
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
    world: &mut World,
    params: &mut SystemState<(EguiContexts, Query<Entity, With<FightWindow>>)>,
) {
    let (ui_ctx, fight_windows) = {
        let (mut egui_contexts, fight_windows) = params.get_mut(world);

        // context for the primary (so far, only) window
        let ui_ctx = match egui_contexts.try_ctx_mut() {
            None => {
                // prevent/avoid a panic when bevy exits.
                // another workaround can be found in https://github.com/mvlabat/bevy_egui/issues/212
                warn!("No egui context, skipping rendering.");
                return;
            }
            // have to make sure not to borrow world. cloning `Context` is cheap.
            Some(ui_ctx) => ui_ctx.clone(),
        };

        // need this owned/non-borrowed as well, so we can still use `world`
        let fight_windows = fight_windows.iter().collect_vec();

        (ui_ctx, fight_windows)
    };

    // enable light style: https://github.com/emilk/egui/discussions/1627
    ui_ctx.style_mut(|style| style.visuals = Visuals::light());

    for fight_window_e in fight_windows.into_iter() {
        egui::Window::new("Fight")
            .id(Id::new(fight_window_e))
            .show(&ui_ctx, |ui: &mut Ui| {
                run_ui_system(
                    ui,
                    world,
                    Id::new("fight_window").with(fight_window_e),
                    fight_window_e,
                    render_fight_window,
                );
            });
    }
}

#[expect(clippy::too_many_arguments)]
pub fn render_fight_window(
    In((mut ui, fight_window_e)): In<(Ui, Entity)>,
    mut fight_windows: Query<&mut FightWindow>,
    fights: Query<(&Fight, Option<&FightResult>)>,
    factions: Query<(Entity, &Faction)>,
    names: Query<&Name>,
    healths: Query<&Health>,
    has_ability_slots: Query<&HasAbilitySlots>,
    has_abilities: Query<&HasAbilities>,
    children: Query<&Children>,
    ability_slots: Query<&AbilitySlot>,
    ability_interface: AbilityInterface,
    mut cast_ability: EventWriter<commands::CastAbility>,
) -> (Ui, ()) {
    let mut fight_window = fight_windows.get_mut(fight_window_e).unwrap();
    let fight_e = fight_window.model;

    let (_, fight_result) = fights
        .get(fight_e)
        .expect("FightWindow.model doesn't have a Fight");

    let fight_children = children.get(fight_e).expect("Fight without Children");
    let player_entity = factions
        .iter_many(fight_children)
        .filter(|(_e, faction)| **faction == Faction::Player)
        .exactly_one()
        .ok() // the error doesn't impl `Debug`, so can't unwrap it
        .unwrap()
        .0;

    let enemy_entity = factions
        .iter_many(fight_children)
        .filter(|(_e, faction)| **faction == Faction::Enemy)
        .exactly_one()
        .ok() // the error doesn't impl `Debug`, so can't unwrap it
        .unwrap()
        .0;

    let ui_state = &mut fight_window.ui_state;

    if let Some(fight_result) = fight_result {
        match fight_result {
            FightResult::FactionVictory { which: win_faction } => {
                ui.vertical_centered(|ui| {
                    ui.label(
                        RichText::new(format!("'{win_faction}' won!"))
                            .heading()
                            .strong(),
                    );
                });
            }
        }
    }
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
            &ability_slots,
            &ability_interface,
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
            &ability_slots,
            &ability_interface,
            &mut cast_ability,
        );
    });

    (ui, ())
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

#[expect(clippy::too_many_arguments)]
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
    ability_slots: &Query<&AbilitySlot>,
    ability_interface: &AbilityInterface,
    cast_ability: &mut EventWriter<commands::CastAbility>,
) {
    ui.indent(ui.id().with("entity_overview_section"), |ui: &mut Ui| {
        if let Ok(name) = names.get(model_e) {
            ui.label(name.as_str());
        } else {
            ui.label("<No Name>");
        }

        if let Ok(health) = healths.get(model_e) {
            ui.label(format!(
                "Health: {:.2}/{:.2}",
                health.current(),
                health.max()
            ));
        } else {
            ui.label("<No Health>");
        }
    });

    if let Ok(slots) = has_ability_slots.get(model_e) {
        ui.add_space(10.);
        ui_ability_slots(
            ui,
            &mut ui_column_state.abilities_section_state,
            slots,
            children,
            ability_slots,
        );
    }

    if let Ok(abilities) = has_abilities.get(model_e) {
        ui.add_space(10.);
        ui_abilities(
            ui,
            model_e,
            fight_e,
            abilities,
            children,
            ability_interface,
            ability_slots,
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

#[expect(clippy::too_many_arguments)]
fn ui_abilities(
    ui: &mut Ui,
    model: Entity,
    fight_e: Entity,
    abilities: &HasAbilities,
    children: &Query<&Children>,
    ability_interface: &AbilityInterface,
    ability_slots: &Query<&AbilitySlot>,
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
            let ability = ability_interface.get_ability_from_entity(*ability_id_e);
            let possible_cast = commands::CastAbility {
                caster_e: model,
                slot_e: selected_slot_e,
                ability_e: *ability_id_e,
                fight_e,
            };
            // TODO: forward/refactor so we have `CastAbilityInterface` here, and use that for
            // checking instead
            let ability_usable = ability.can_use_slot(selected_slot);

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
                        cast_ability.send(possible_cast);

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
        Some(shortcut) => ui.input_mut(|i| i.consume_shortcut(shortcut)),
        None => false,
    };

    let shortcut_text: String = match shortcut {
        Some(shortcut) => ui.ctx().format_shortcut(shortcut),
        None => String::from(" "),
    };

    ui.monospace(shortcut_text);

    shortcut_pressed
}
