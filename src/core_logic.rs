use std::borrow::Cow;

use bevy::{prelude::*, utils::Duration};
use smallvec::SmallVec;

#[derive(Debug, Clone, Component, Reflect)]
pub struct Fight {
    pub player_character: Entity,
    pub enemy: Entity,
}

#[derive(Debug, Component, Reflect)]
pub struct PlayerCharacter;

#[derive(Debug, Component, Reflect)]
pub struct Enemy;

#[derive(Debug, Component, Reflect)]
pub struct AbilitySlots(pub SmallVec<[AbilitySlot; 4]>);

#[derive(Debug, Component, Reflect)]
pub struct HasAbilities(pub SmallVec<[AbilityId; 4]>);

#[derive(Debug, Reflect)]
pub struct AbilitySlot {
    pub tpe: AbilitySlotType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect)]
pub enum AbilitySlotType {
    WeaponAttack,
    ShieldDefend,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Reflect)]
pub enum AbilityId {
    Attack,
}

#[derive(Debug)]
pub struct Ability {
    pub name: Cow<'static, str>,
    pub id: AbilityId,
    pub slot_type: AbilitySlotType,
    pub cooldown: Duration,
}

pub struct CoreLogicPlugin;

impl Plugin for CoreLogicPlugin {
    fn build(&self, app: &mut App) {
        // from https://github.com/jakobhellermann/bevy-inspector-egui/discussions/130
        app.register_type::<Fight>()
            .register_type::<Enemy>()
            .register_type::<HasAbilities>()
            .register_type::<AbilitySlots>()
            .register_type::<PlayerCharacter>();
    }
}
