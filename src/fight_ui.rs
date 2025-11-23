use bevy::{ecs::entity::EntityHashSet, prelude::*};
use bevy_inspector_egui::bevy_egui::EguiPrimaryContextPass;
use render_effects::RenderEffectsPlugin;
use render_fight_window::{FightWindowUiState, render_fight_windows};

use crate::{PerUpdateSet, game_logic::fight::Fight};

mod render_effects;
mod render_fight_window;

pub struct FightUiPlugin;

impl Plugin for FightUiPlugin {
    fn build(&self, app: &mut App) {
        app
            //
            .add_plugins(RenderEffectsPlugin)
            .add_systems(
                EguiPrimaryContextPass,
                (sync_fight_windows_to_fights, render_fight_windows)
                    .chain()
                    .in_set(PerUpdateSet::CommandSubmission),
            );
    }
}

#[derive(Debug, Component, Reflect)]
struct FightWindow {
    model: Entity,
    ui_state: FightWindowUiState,
}

impl FightWindow {
    fn new(model: Entity) -> Self {
        Self {
            model,
            ui_state: FightWindowUiState::default(),
        }
    }
}

// Create `FightWindow`s for new fights, and delete `FightWindow`s for removed fights.
fn sync_fight_windows_to_fights(
    mut commands: Commands,
    new_fights: Query<Entity, Added<Fight>>,
    fight_windows: Query<(Entity, &FightWindow)>,
    mut removed_fights: RemovedComponents<Fight>,
) {
    for fight_e in new_fights.iter() {
        commands.spawn((FightWindow::new(fight_e),));
    }

    let mut removed_fight_entities: EntityHashSet = removed_fights.read().collect();

    if !removed_fight_entities.is_empty() {
        for (window_e, fight_window) in fight_windows.iter() {
            let should_despawn = removed_fight_entities.remove(&fight_window.model);

            if should_despawn {
                commands.entity(window_e).despawn();
            }
        }
    }
}
