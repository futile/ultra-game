use std::borrow::Cow;

use bevy::{ecs::system::SystemState, prelude::*};
use bevy_inspector_egui::{
    bevy_egui::EguiContexts,
    egui::{self, Id, Key, KeyboardShortcut, Modifiers, RichText, Ui, Visuals},
};
use itertools::Itertools;

use super::FightWindow;
use crate::{
    abilities::AbilityInterface,
    game_logic::{
        commands::{self, CastAbilityInterface, GameCommand},
        faction::Faction,
        fight::{Fight, FightInterface, FightResult, FightTime},
        health::Health,
        Ability, AbilitySlot,
    },
    utils::egui_systems::run_ui_system,
    AbilitySlotType, HasAbilities, HasAbilitySlots,
};

#[derive(Debug, Clone, Component, Reflect)]
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

        // in case we ever have things that need to be applied, e.g., `Commands`.
        // should be done when we are done with the `SystemState`.
        // should this be done later? I would think it doesn't hurt to do it here..
        params.apply(world);

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

pub fn render_fight_window(
    In((mut ui, fight_window_e)): In<(Ui, Entity)>,
    world: &mut World,
    fight_windows: &mut QueryState<&mut FightWindow>,
    fights: &mut QueryState<(&Fight, &mut FightTime, Option<&FightResult>)>,
    factions: &mut QueryState<(Entity, &Faction)>,
    children: &mut QueryState<&Children>,
) -> (Ui, ()) {
    let fight_window = fight_windows.get_mut(world, fight_window_e).unwrap();

    let fight_e = fight_window.model;
    let mut ui_state = fight_window.ui_state.clone();

    let (_fight, fight_time, fight_result) = fights
        .get(world, fight_e)
        .expect("FightWindow.model doesn't have a Fight");

    let fight_children = children
        .get(world, fight_e)
        .expect("Fight without Children");
    let player_entity = factions
        .iter_many(world, fight_children)
        .filter(|(_e, faction)| **faction == Faction::Player)
        .exactly_one()
        .ok() // the error doesn't impl `Debug`, so can't unwrap it
        .unwrap()
        .0;

    let enemy_entity = factions
        .iter_many(world, fight_children)
        .filter(|(_e, faction)| **faction == Faction::Enemy)
        .exactly_one()
        .ok() // the error doesn't impl `Debug`, so can't unwrap it
        .unwrap()
        .0;

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

    let pause_toggled = {
        let elapsed = fight_time.stop_watch.elapsed();

        let minutes = elapsed.as_secs() / 60;
        let secs = elapsed.as_secs() % 60;
        let tenths = elapsed.subsec_millis() / 100;

        let play_pause_interactable = fight_result.is_none();
        let mut pause_toggled = false;

        ui.vertical_centered(|ui| {
            ui.allocate_ui_with_layout(
                // x == 100.0 seems to.. just work here.
                // tried a lot, but can't do much better, this is one of the prominent weaknesses
                // of immediate mode GUIs.
                egui::Vec2::new(100.0, 0.0),
                egui::Layout::left_to_right(egui::Align::Center),
                |ui| {
                    ui.add_enabled_ui(play_pause_interactable, |ui| {
                        let space_pressed = ui.input_mut(|i| {
                            i.consume_shortcut(&KeyboardShortcut::new(Modifiers::NONE, Key::Space))
                        });
                        let timer_clicked = ui
                            .button(
                                RichText::new(format!("{minutes:02}:{secs:02}.{tenths:1}"))
                                    .heading()
                                    .strong(),
                            )
                            .clicked();
                        pause_toggled = space_pressed || timer_clicked;
                    });
                },
            );
        });

        pause_toggled
    };

    ui.columns(2, |columns: &mut [Ui]| {
        columns[0].label(RichText::new("Player").heading().strong());

        ui_state.player_column_state = run_ui_system(
            &mut columns[0],
            world,
            Id::new("fight_column")
                .with(fight_window_e)
                .with(player_entity),
            (ui_state.player_column_state.clone(), player_entity, fight_e),
            ui_fight_column,
        );

        columns[1].label(RichText::new("Enemy").heading().strong());

        ui_state.enemy_column_state = run_ui_system(
            &mut columns[1],
            world,
            Id::new("fight_column")
                .with(fight_window_e)
                .with(enemy_entity),
            (ui_state.enemy_column_state.clone(), enemy_entity, fight_e),
            ui_fight_column,
        );
    });

    fight_windows
        .get_mut(world, fight_window_e)
        .unwrap()
        .ui_state = ui_state;

    if pause_toggled {
        let (_fight, mut fight_time, _fight_result) = fights
            .get_mut(world, fight_e)
            .expect("FightWindow.model doesn't have a Fight");

        if fight_time.stop_watch.paused() {
            fight_time.stop_watch.unpause();
        } else {
            fight_time.stop_watch.pause();
        }
    }

    (ui, ())
}

#[derive(Debug, Clone, Reflect)]
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

#[derive(Debug, Clone, Reflect)]
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
    In((mut ui, (mut ui_column_state, model_e, fight_e))): In<(
        Ui,
        (FightColumnUiState, Entity, Entity),
    )>,
    world: &mut World,
    names: &mut QueryState<&Name>,
    healths: &mut QueryState<&Health>,
    has_ability_slots: &mut QueryState<&HasAbilitySlots>,
    has_abilities: &mut QueryState<&HasAbilities>,
) -> (Ui, FightColumnUiState) {
    ui.indent(ui.id().with("entity_overview_section"), |ui: &mut Ui| {
        if let Ok(name) = names.get(world, model_e) {
            ui.label(name.as_str());
        } else {
            ui.label("<No Name>");
        }

        if let Ok(health) = healths.get(world, model_e) {
            ui.label(format!(
                "Health: {:.2}/{:.2}",
                health.current(),
                health.max()
            ));
        } else {
            ui.label("<No Health>");
        }
    });

    if has_ability_slots.get(world, model_e).is_ok() {
        ui.add_space(10.);

        ui_column_state.abilities_section_state = run_ui_system(
            &mut ui,
            world,
            Id::new("slots_section").with(model_e),
            (
                model_e,
                fight_e,
                ui_column_state.abilities_section_state.clone(),
            ),
            ui_ability_slots,
        );
    }

    if has_abilities.get(world, model_e).is_ok() {
        ui.add_space(10.);

        ui_column_state = run_ui_system(
            &mut ui,
            world,
            Id::new("abilities_section").with(model_e),
            (model_e, fight_e, ui_column_state.clone()),
            ui_abilities,
        );
    }

    (ui, ui_column_state)
}

#[expect(
    clippy::type_complexity,
    reason = "SystemState<..> big but ok, part of the ui-pattern (for now)"
)]
fn ui_ability_slots(
    In((mut ui, (model_e, fight_e, mut slots_section_state))): In<(
        Ui,
        (Entity, Entity, AbilitySlotsSectionUiState),
    )>,
    world: &mut World,
    params: &mut SystemState<(
        Query<&HasAbilitySlots>,
        Query<&Children>,
        Query<&AbilitySlot>,
        FightInterface,
    )>,
) -> (Ui, AbilitySlotsSectionUiState) {
    // TODO: add colors (again) at some point (if it fits..)
    // old colors for reference:
    // AbilitySlotType::WeaponAttack => Color::LIME_GREEN,
    // AbilitySlotType::ShieldDefend => Color::PINK,

    {
        let (slots, children, ability_slots, fight_interface) = params.get_mut(world);

        let user_interactable = slots_section_state.user_interactable
            && !fight_interface.get_fight_status(fight_e).is_ended();

        ui.heading("Ability Slots");

        let slots_holder = slots.get(model_e).unwrap().holder;

        ui.indent(ui.id().with("ability_slots"), |ui: &mut Ui| {
            for (idx, &slot_e) in children
                .get(slots_holder)
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

                let slot_is_selected: bool = slots_section_state
                    .selected_slot
                    .is_some_and(|s| s == slot_e);

                ui.horizontal(|ui: &mut Ui| {
                    let shortcut_pressed =
                        monospace_checked_shortcut(ui, keyboard_shortcut.as_ref());

                    let mut label_response = ui
                        .add_enabled_ui(user_interactable, |ui: &mut Ui| {
                            ui.selectable_label(slot_is_selected, text_for_slot_type(&slot.tpe))
                        })
                        .inner;

                    if shortcut_pressed || label_response.clicked() {
                        slots_section_state.selected_slot =
                            if slot_is_selected { None } else { Some(slot_e) };

                        // not 100% sure why this is needed, but `Ui::selectable_value()` does it as
                        // well, so it might be necessary.
                        label_response.mark_changed();
                    }
                });
            }
        });
    }

    params.apply(world);

    (ui, slots_section_state)
}

#[expect(
    clippy::type_complexity,
    reason = "SystemState<..> big but ok, part of the ui-pattern (for now)"
)]
fn ui_abilities(
    In((mut ui, (model_e, fight_e, mut ui_column_state))): In<(
        Ui,
        (Entity, Entity, FightColumnUiState),
    )>,
    world: &mut World,
    params: &mut SystemState<(
        Query<&HasAbilities>,
        Query<&Children>,
        AbilityInterface,
        CastAbilityInterface,
        EventWriter<GameCommand>,
    )>,
) -> (Ui, FightColumnUiState) {
    {
        #[rustfmt::skip]
        let (
            has_abilities,
            children,
            ability_interface,
            cast_ability_interface,
            mut game_commands,
        ) = params.get_mut(world);

        let user_interactable = ui_column_state.user_interactable;
        let selected_slot_e = ui_column_state.abilities_section_state.selected_slot;

        ui.heading("Abilities");

        let abilities = has_abilities.get(model_e).unwrap();

        ui.indent(ui.id().with("abilities"), |ui: &mut Ui| {
            for (idx, ability_id_e) in children
                .get(abilities.holder)
                .expect("HasAbilities.holder without Children")
                .iter()
                .enumerate()
            {
                let ability = ability_interface.get_ability_from_entity(*ability_id_e);
                let possible_cast = commands::CastAbility {
                    caster_e: model_e,
                    slot_e: selected_slot_e,
                    ability_e: *ability_id_e,
                    fight_e,
                };
                let ability_usable = cast_ability_interface.is_valid_cast(&possible_cast);

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
                            egui::Button::new(ability.name.clone()),
                        );

                        // `hovered()`, `show_tooltip_at_pointer()`, etc., all don't work when
                        // disabled. but we wan't tooltips for disabled abilities as well.
                        if ability_button.contains_pointer() {
                            // tooltip at pointer is not ideal, e.g., moves with pointer, but also
                            // overlaps with my huge-size cursor.
                            // See also: https://github.com/rust-windowing/winit/issues/3788
                            // egui::containers::popup::show_tooltip_at_pointer(
                            //     ui.ctx(),
                            //     ui.layer_id(),
                            //     Id::new("AbilityTooltip").with(idx),
                            //     tooltip_for_ability(ability.clone()),
                            // );

                            // show the tooltip next to the button, i.e., to the right-side of it.
                            egui::containers::popup::show_tooltip_at(
                                ui.ctx(),
                                ui.layer_id(),
                                Id::new("AbilityTooltip").with(idx),
                                ability_button.rect.right_top(),
                                tooltip_for_ability(ability.clone()),
                            );
                        }

                        if ability_usable && (shortcut_pressed || ability_button.clicked()) {
                            game_commands.send(GameCommand::new_from_user(possible_cast.into()));

                            // clear the selected slot, because it was used.
                            ui_column_state.abilities_section_state.selected_slot = None;
                        }
                    });
                });
            }
        });
    }

    params.apply(world);

    (ui, ui_column_state)
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

fn tooltip_for_ability(ability: Ability) -> impl FnOnce(&mut Ui) {
    move |ui| {
        ui.label(format!(
            "Required Slot: {}\n", // newline for spacing
            text_for_slot_type(&ability.slot_type)
        ));

        ui.label(ability.description.clone());
    }
}

fn text_for_slot_type(slot_type: &AbilitySlotType) -> Cow<'static, str> {
    match slot_type {
        AbilitySlotType::WeaponAttack => Cow::from("Weapon Attack"),
        AbilitySlotType::ShieldDefend => Cow::from("Shield Defend"),
    }
}
