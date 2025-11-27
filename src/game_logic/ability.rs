use std::borrow::Cow;

use bevy::prelude::*;

use crate::{
    game_logic::ability_slots::AbilitySlotType,
    utils::holds_held::{Held, Holds},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Reflect)]
pub enum AbilityId {
    WeaponAttack,
    NeedlingHex,
    ChargedStrike,
}

#[derive(Debug, Clone, Component, Reflect)]
pub struct Ability {
    pub id: AbilityId,
    pub name: Cow<'static, str>,
    pub description: Cow<'static, str>,
}

#[derive(Debug, Clone, Component, Reflect)]
pub struct AbilitySlotRequirement(pub AbilitySlotType);

#[derive(Debug, Clone, Component, Reflect, Default)]
pub struct AbilityCooldown {
    pub duration: std::time::Duration,
}

#[derive(Debug, Clone, Component, Reflect)]
pub struct AbilityCastTime(pub std::time::Duration);

#[derive(EntityEvent, Debug, Reflect)]
pub struct PerformAbility {
    #[event_target]
    pub ability_entity: Entity,
    pub target: Option<Entity>,
    pub slot: Entity,
}

#[derive(Component, Debug, Reflect, Default)]
pub struct CastFailed<T: Send + Sync + 'static> {
    #[reflect(ignore)]
    pub _marker: std::marker::PhantomData<T>,
}

pub struct AbilityPlugin;

impl Plugin for AbilityPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Holds<Ability>>()
            .register_type::<Held<Ability>>()
            .register_type::<Ability>()
            .register_type::<AbilitySlotRequirement>()
            .register_type::<AbilityCooldown>()
            .register_type::<AbilityCastTime>()
            .register_type::<PerformAbility>();
    }
}
