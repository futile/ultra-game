use bevy::prelude::*;

pub mod ability;
pub mod ability_casting;
pub mod ability_slots;
pub mod commands;
pub mod cooldown;
pub mod damage_resolution;
pub mod effects;
pub mod faction;
pub mod fight;
pub mod health;
pub mod ongoing_cast;

pub struct GameLogicPlugin;

impl Plugin for GameLogicPlugin {
    fn build(&self, app: &mut App) {
        // from https://github.com/jakobhellermann/bevy-inspector-egui/discussions/130
        app.add_plugins((
            ability::AbilityPlugin,
            ability_slots::AbilitySlotsPlugin,
            commands::CommandsPlugin,
            cooldown::CooldownPlugin,
            damage_resolution::DamageResolutionPlugin,
            effects::EffectsPlugin,
            faction::FactionPlugin,
            fight::FightPlugin,
            health::HealthInterfacePlugin,
            ongoing_cast::OngoingCastPlugin,
        ));
    }
}
