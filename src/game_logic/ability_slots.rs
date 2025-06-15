use bevy::prelude::*;

#[derive(Debug, Component, Reflect)]
pub struct AbilitySlot {
    pub tpe: AbilitySlotType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect)]
pub enum AbilitySlotType {
    WeaponAttack,
    ShieldDefend,
}

#[derive(Debug, Component, Reflect)]
pub struct HasAbilitySlots {
    pub holder: Entity,
}

pub struct AbilitySlotsPlugin;

impl Plugin for AbilitySlotsPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<HasAbilitySlots>()
            .register_type::<AbilitySlot>();
    }
}
