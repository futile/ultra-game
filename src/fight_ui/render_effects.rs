use std::{fmt::Write as _, sync::LazyLock, time::Duration};

use bevy::prelude::*;
use bevy_inspector_egui::egui::{self, Id, Ui};

use crate::{abilities::needling_hex::NeedlingHexEffect, utils::SplitDuration};

#[reflect_trait]
pub trait RenderGameEffectImmediate {
    fn render_to_ui(&self, ui: &mut Ui);
}

impl RenderGameEffectImmediate for NeedlingHexEffect {
    fn render_to_ui(&self, ui: &mut Ui) {
        static DMG_DESCRIPTION: LazyLock<String> = LazyLock::new(|| {
            format!(
                "Deals {dmg_per_tick} damage every {tick_interval}s, a total of {num_ticks} times.",
                dmg_per_tick = NeedlingHexEffect::DMG_PER_TICK,
                tick_interval = NeedlingHexEffect::TICK_INTERVAL.as_secs_f64(),
                num_ticks = NeedlingHexEffect::NUM_TICKS,
            )
        });

        let label = ui.label(format!(
            "{remaining_time} Needling Hex ({remaining_ticks})",
            remaining_time = format_remaining_time(&self.remaining_time()),
            remaining_ticks = self.remaining_ticks()
        ));

        if label.contains_pointer() {
            egui::Tooltip::always_open(
                ui.ctx().clone(),
                ui.layer_id(),
                Id::new("EffectTooltip").with(self as *const _),
                label.rect.right_top(),
            )
            .show(|ui| {
                ui.label("A maddening hex that causes you to repeatedly take damage.");
                ui.label("");
                ui.label(&*DMG_DESCRIPTION);
            });
        }
    }
}

pub fn format_remaining_time(remaining: &Duration) -> String {
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
