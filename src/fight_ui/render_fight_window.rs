use std::{borrow::Cow, fmt::Write as _};

use bevy::{ecs::system::SystemState, prelude::*};
use bevy_inspector_egui::{
    bevy_egui::EguiContexts,
    egui::{
        self, Id, Key, KeyboardShortcut, Modifiers, ProgressBar, RichText, Ui, Visuals, Widget,
    },
};
use itertools::Itertools;

use super::{
    render_effects::{format_remaining_time, ReflectRenderGameEffectImmediate},
    FightWindow,
};
use crate::{
    abilities::AbilityInterface,
    game_logic::{
        ability::{Ability, AbilityId},
        ability_slots::{AbilitySlot, AbilitySlotType},
        commands::{self, CastAbilityInterface, GameCommand},
        effects::{HasEffects, ReflectGameEffect},
        faction::Faction,
        fight::{Fight, FightInterface, FightResult, FightTime},
        health::Health,
        ongoing_cast::OngoingCastInterface,
    },
    utils::{egui_systems::run_ui_system, holds_held::Holds, SplitDuration},
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
            .default_size((500.0, 500.0))
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
    fight_interface: &mut SystemState<FightInterface>,
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
        let timer_string: String = {
            let elapsed = fight_time.stop_watch().elapsed();
            let elapsed_split = SplitDuration::from_duration(&elapsed);

            let mut s = String::new();

            if elapsed_split.days > 0 {
                write!(&mut s, "{}d, ", elapsed_split.days).unwrap();
            }

            if elapsed_split.hours > 0 {
                write!(&mut s, "{:02}:", elapsed_split.hours).unwrap();
            }

            write!(
                &mut s,
                "{:02}:{:02}.{:1}",
                elapsed_split.minutes, elapsed_split.seconds, elapsed_split.tenths
            )
            .unwrap();

            s
        };

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
                            .button(RichText::new(timer_string).heading().strong())
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
        let mut fight_interface = fight_interface.get_mut(world);
        let is_paused = fight_interface.is_fight_paused(fight_e);

        fight_interface.set_fight_paused(fight_e, !is_paused);
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
    holds_ability_slots: &mut QueryState<&Holds<AbilitySlot>>,
    holds_ability_ids: &mut QueryState<&Holds<AbilityId>>,
    has_effects: &mut QueryState<&HasEffects>,
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

    if holds_ability_slots.get(world, model_e).is_ok() {
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

    if holds_ability_ids.get(world, model_e).is_ok() {
        ui.add_space(10.);

        ui_column_state = run_ui_system(
            &mut ui,
            world,
            Id::new("abilities_section").with(model_e),
            (model_e, fight_e, ui_column_state.clone()),
            ui_abilities,
        );
    }

    if has_effects.get(world, model_e).is_ok() {
        ui.add_space(10.);

        run_ui_system(
            &mut ui,
            world,
            Id::new("effects_section").with(model_e),
            (model_e,),
            ui_effects,
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
        Query<&Holds<AbilitySlot>>,
        Query<&AbilitySlot>,
        FightInterface,
        AbilityInterface,
        OngoingCastInterface,
    )>,
) -> (Ui, AbilitySlotsSectionUiState) {
    // TODO: add colors (again) at some point (if it fits..)
    // old colors for reference:
    // AbilitySlotType::WeaponAttack => Color::LIME_GREEN,
    // AbilitySlotType::ShieldDefend => Color::PINK,

    {
        let (slots, ability_slots, fight_interface, ability_interface, ongoing_cast_interface) =
            params.get_mut(world);

        let user_interactable = slots_section_state.user_interactable
            && !fight_interface.get_fight_status(fight_e).is_ended();

        ui.heading("Ability Slots");

        ui.indent(ui.id().with("ability_slots"), |ui: &mut Ui| {
            for (idx, slot_e) in slots.relationship_sources(model_e).enumerate() {
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

                if let Some(ongoing_cast) = ongoing_cast_interface.get_ongoing_cast(slot_e) {
                    let progress = 1.0 - ongoing_cast.cast_timer.fraction_remaining();
                    let remaining = ongoing_cast.cast_timer.remaining();
                    let ability = ability_interface.get_ability_from_entity(ongoing_cast.ability_e);

                    ui.indent(Id::new("progress_bar_for_slot").with(slot_e), |ui| {
                        let progress_text = format!(
                            "{} - {}",
                            ability.name.clone(),
                            format_remaining_time(&remaining),
                        );

                        ProgressBar::new(progress)
                            .animate(false) // animates a spinner, didn't like it super much.
                            .text(progress_text)
                            .ui(ui);
                    });
                }
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
        Query<&Holds<AbilityId>>,
        AbilityInterface,
        CastAbilityInterface,
        EventWriter<GameCommand>,
    )>,
) -> (Ui, FightColumnUiState) {
    {
        #[rustfmt::skip]
        let (
            holds_ability_ids,
            ability_interface,
            cast_ability_interface,
            mut game_commands,
        ) = params.get_mut(world);

        let user_interactable = ui_column_state.user_interactable;
        let selected_slot_e = ui_column_state.abilities_section_state.selected_slot;

        ui.heading("Abilities");

        ui.indent(ui.id().with("abilities"), |ui: &mut Ui| {
            for (idx, ability_id_e) in holds_ability_ids.relationship_sources(model_e).enumerate() {
                let ability = ability_interface.get_ability_from_entity(ability_id_e);
                
                let (slot_e, ability_usable) = if let Some(slot_e) = selected_slot_e {
                    let possible_cast = commands::UseAbility {
                        caster_e: model_e,
                        slot_e,
                        ability_e: ability_id_e,
                        fight_e,
                    };
                    let ability_usable = cast_ability_interface.is_valid_cast(&possible_cast);
                    (slot_e, ability_usable)
                } else {
                    // No slot selected, ability cannot be used
                    // Use a dummy entity ID for slot_e since we won't actually cast
                    (Entity::PLACEHOLDER, false)
                };

                let keyboard_shortcut: Option<KeyboardShortcut> = if user_interactable {
                    let key: Option<Key> = match idx {
                        0 => Some(Key::X),
                        1 => Some(Key::V),
                        2 => Some(Key::L),
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
                            let cast_command = commands::UseAbility {
                                caster_e: model_e,
                                slot_e,
                                ability_e: ability_id_e,
                                fight_e,
                            };
                            game_commands.write(GameCommand::new_from_user(cast_command.into()));

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

fn ui_effects(
    In((mut ui, (model_e,))): In<(Ui, (Entity,))>,
    world: &mut World,
    params: &mut SystemState<(Query<&HasEffects>, Query<&Children>, Res<AppTypeRegistry>)>,
) -> (Ui, ()) {
    ui.heading("Effects");

    let (effect_entities, app_type_registry) = {
        let (has_effects, children, world_type_registry) = params.get_mut(world);
        let holder = has_effects.get(model_e).unwrap().holder();
        let children = children
            .get(holder)
            .ok()
            .map(|cs| cs.to_vec())
            .unwrap_or_default();

        let app_type_registry = world_type_registry.clone();

        params.apply(world);

        (children, app_type_registry)
    };

    let type_registry = app_type_registry.read();

    for effect_e in effect_entities {
        let component_infos = world.inspect_entity(effect_e).unwrap();
        for component_info in component_infos {
            let Some(component_type_id) = component_info.type_id() else {
                warn_once!(
                    "Component `{}` does not have a type_id()!",
                    component_info.name()
                );
                continue;
            };

            let comp_as_reflect = match world.get_reflect(effect_e, component_type_id) {
                Ok(reflect) => reflect,
                Err(e) => {
                    warn_once!("Could not get effect entity as Reflect: {e}");
                    continue;
                }
            };

            let Some(_reflect_game_effect) =
                type_registry.get_type_data::<ReflectGameEffect>(component_type_id)
            else {
                // this is not really a warning/error, as, e.g., the `Parent` component will be
                // present but doesn't impl `GameEffect`.
                // warn_once!(
                //     "Component `{}` is not ReflectGameEffect!",
                //     component_info.name()
                // );
                continue;
            };

            let Some(reflect_render_game_effect_immediate) =
                type_registry.get_type_data::<ReflectRenderGameEffectImmediate>(component_type_id)
            else {
                warn_once!(
                    "Component `{}` is ReflectGameEffect but not ReflectRenderGameEffectImmediate!",
                    component_info.name()
                );

                let short_name = component_info
                    .name()
                    .rsplit_once(':')
                    .map(|(_prefix, shortname)| shortname)
                    .unwrap_or(component_info.name());

                let label = ui.label(format!("[UNKNOWN] {short_name}"));

                if label.contains_pointer() {
                    egui::containers::popup::show_tooltip_at(
                        ui.ctx(),
                        ui.layer_id(),
                        Id::new("UnknownEffectTooltip").with(comp_as_reflect as *const _),
                        label.rect.right_top(),
                        |ui| {
                            ui.label(component_info.name());
                        },
                    );
                }

                continue;
            };

            let comp_as_render_game_effect_immediate = reflect_render_game_effect_immediate
                .get(comp_as_reflect)
                .unwrap();

            comp_as_render_game_effect_immediate.render_to_ui(&mut ui);
        }
    }

    (ui, ())
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
        if let Some(required_slot_type) = ability.slot_type {
            ui.label(format!(
                "Required Slot: {}\n", // newline for spacing
                text_for_slot_type(&required_slot_type)
            ));
        }

        ui.label(ability.description.clone());
    }
}

fn text_for_slot_type(slot_type: &AbilitySlotType) -> Cow<'static, str> {
    match slot_type {
        AbilitySlotType::WeaponAttack => Cow::from("Weapon Attack"),
        AbilitySlotType::ShieldDefend => Cow::from("Shield Defend"),
        AbilitySlotType::Magic => Cow::from("Magic"),
    }
}
