use bevy::prelude::*;

use self::render_fight_window::ui_render_fight_windows;

mod render_fight_window;

pub struct FightUiPlugin;

impl Plugin for FightUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, ui_render_fight_windows);
    }
}
