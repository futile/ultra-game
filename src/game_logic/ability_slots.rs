use bevy::prelude::*;

use crate::utils::holds_held::{Held, Holds};

#[derive(Debug, Component, Reflect)]
pub struct AbilitySlot {
    pub tpe: AbilitySlotType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect)]
pub enum AbilitySlotType {
    WeaponAttack,
    ShieldDefend,
    Magic,
}

pub struct AbilitySlotsPlugin;

impl Plugin for AbilitySlotsPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Holds<AbilitySlot>>()
            .register_type::<Held<AbilitySlot>>()
            .register_type::<AbilitySlot>();
    }
}
