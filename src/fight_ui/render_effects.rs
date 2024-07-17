use std::{fmt::Write as _, time::Duration};

use bevy::prelude::*;
use bevy_inspector_egui::egui::Ui;

use crate::{abilities::needling_hex::NeedlingHexEffect, utils::SplitDuration};

#[reflect_trait]
pub trait RenderGameEffectImmediate {
    fn render_to_ui(&self, ui: &mut Ui);
}

impl RenderGameEffectImmediate for NeedlingHexEffect {
    fn render_to_ui(&self, ui: &mut Ui) {
        ui.label(format!(
            "{remaining_time} Needling Hex ({remaining_ticks})",
            remaining_time = format_remaining_effect_time(&self.remaining_time()),
            remaining_ticks = self.remaining_ticks()
        ));
    }
}

fn format_remaining_effect_time(remaining: &Duration) -> String {
    let SplitDuration {
        days,
        hours,
        minutes,
        seconds,
        tenths,
        hundredths: _,
        millis: _,
    } = SplitDuration::from_duration(remaining);

    let mut s = String::new();

    if days > 0 {
        write!(&mut s, "{days}d").unwrap();
    } else if hours > 0 {
        write!(&mut s, "{hours}h").unwrap();
    } else if minutes > 0 {
        write!(&mut s, "{minutes}m").unwrap();
    } else if seconds >= 10 {
        write!(&mut s, "{seconds}s").unwrap();
    } else {
        write!(&mut s, "{seconds}.{tenths}s").unwrap();
    }

    s
}

pub struct RenderEffectsPlugin;

impl Plugin for RenderEffectsPlugin {
    fn build(&self, app: &mut App) {
        app.register_type_data::<NeedlingHexEffect, ReflectRenderGameEffectImmediate>();
    }
}
