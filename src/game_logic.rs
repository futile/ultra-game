use std::borrow::Cow;

use bevy::prelude::*;

pub mod commands;
pub mod damage_resolution;
pub mod faction;

#[derive(Debug, Clone, Component, Reflect)]
pub struct Fight;

#[derive(Debug, Clone, Component, Reflect)]
pub struct Health {
    pub current: f64,
    pub max: f64,
}

impl Health {
    pub fn new(current_and_max: f64) -> Self {
        Self {
            current: current_and_max,
            max: current_and_max,
        }
    }
}

#[derive(Debug, Component, Reflect)]
pub struct HasAbilitySlots {
    pub holder: Entity,
}

#[derive(Debug, Component, Reflect)]
pub struct HasAbilities {
    pub holder: Entity,
}

#[derive(Debug, Component, Reflect)]
pub struct AbilitySlot {
    pub tpe: AbilitySlotType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect)]
pub enum AbilitySlotType {
    WeaponAttack,
    ShieldDefend,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, Hash, Reflect)]
pub enum AbilityId {
    Attack,
}

#[derive(Debug, Reflect)]
pub struct Ability {
    pub name: Cow<'static, str>,
    pub id: AbilityId,
    pub slot_type: AbilitySlotType,
}

impl Ability {
    pub fn can_use(&self, selected_ability_slot: Option<&AbilitySlot>) -> bool {
        selected_ability_slot.is_some_and(|s| s.tpe == self.slot_type)
    }
}

pub struct GameLogicPlugin;

impl Plugin for GameLogicPlugin {
    fn build(&self, app: &mut App) {
        // from https://github.com/jakobhellermann/bevy-inspector-egui/discussions/130
        app.register_type::<Fight>()
            .register_type::<Health>()
            .register_type::<HasAbilities>()
            .register_type::<AbilityId>()
            .register_type::<HasAbilitySlots>()
            .add_plugins((
                faction::FactionPlugin,
                commands::CommandsPlugin,
                damage_resolution::DamageResolutionPlugin,
            ));
    }
}
