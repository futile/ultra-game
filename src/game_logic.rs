use std::borrow::Cow;

use bevy::prelude::*;

pub mod commands;
pub mod damage_resolution;
pub mod effects;
pub mod faction;
pub mod fight;
pub mod health;

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
}

#[derive(Debug, Clone, Reflect)]
pub struct Ability {
    pub name: Cow<'static, str>,
    pub id: AbilityId,
    pub slot_type: Option<AbilitySlotType>,
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

pub struct GameLogicPlugin;

impl Plugin for GameLogicPlugin {
    fn build(&self, app: &mut App) {
        // from https://github.com/jakobhellermann/bevy-inspector-egui/discussions/130
        app.register_type::<HasAbilities>()
            .register_type::<AbilityId>()
            .register_type::<HasAbilitySlots>()
            .add_plugins((
                fight::FightPlugin,
                faction::FactionPlugin,
                effects::EffectsPlugin,
                commands::CommandsPlugin,
                damage_resolution::DamageResolutionPlugin,
                health::HealthInterfacePlugin,
            ));
    }
}
