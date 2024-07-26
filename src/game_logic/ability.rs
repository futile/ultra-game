use std::{borrow::Cow, time::Duration};

use bevy::prelude::*;

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
    NeedlingHex,
    ChargedStrike,
}

#[derive(Debug, Clone, Reflect)]
pub struct Ability {
    pub name: Cow<'static, str>,
    pub id: AbilityId,
    pub slot_type: Option<AbilitySlotType>,
    pub cast_time: Option<Duration>,
    pub description: Cow<'static, str>,
}

impl Ability {
    pub fn can_use_slot(&self, selected_ability_slot: Option<&AbilitySlot>) -> bool {
        match (self.slot_type, selected_ability_slot) {
            (Some(self_tpe), Some(selected_slot)) => selected_slot.tpe == self_tpe,
            (None, None) => true,
            (Some(_), None) | (None, Some(_)) => false,
        }
    }
}

pub struct AbilityPlugin;

impl Plugin for AbilityPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<HasAbilities>()
            .register_type::<AbilityId>()
            .register_type::<HasAbilitySlots>();
    }
}
