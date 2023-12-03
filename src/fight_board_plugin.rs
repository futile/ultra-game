use bevy::prelude::*;

mod abilities_section;
mod ability_slots_section;
mod fight_added;

pub struct FightBoardPlugin;

impl Plugin for FightBoardPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                fight_added::fight_added,
                ability_slots_section::sync_to_models,
                abilities_section::sync_to_models,
            )
                .chain(),
        );
    }
}
