use bevy::prelude::*;
use bevy_inspector_egui::egui::Ui;

use crate::abilities::needling_hex::NeedlingHexEffect;

#[reflect_trait]
pub trait RenderGameEffectImmediate {
    fn render_to_ui(&self, ui: &mut Ui);
}

impl RenderGameEffectImmediate for NeedlingHexEffect {
    fn render_to_ui(&self, ui: &mut Ui) {
        ui.label(format!("Needling Hex ({})", self.remaining_ticks()));
    }
}

pub struct RenderEffectsPlugin;

impl Plugin for RenderEffectsPlugin {
    fn build(&self, app: &mut App) {
        app.register_type_data::<NeedlingHexEffect, ReflectRenderGameEffectImmediate>();
    }
}
