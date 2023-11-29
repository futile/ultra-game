use bevy::prelude::*;
use smallvec::SmallVec;

#[derive(Debug, Clone, Component, Reflect)]
pub struct Fight {
    pub player_character: Entity,
    pub enemy: Entity,
}

#[derive(Debug, Component, Reflect)]
pub struct PlayerCharacter {
    pub character: Character,
}

#[derive(Debug, Component, Reflect)]
pub struct Enemy;

#[derive(Debug, Reflect)]
pub struct Character {
    pub slots: SmallVec<[AbilitySlot; 4]>,
}

#[derive(Debug, Reflect)]
pub struct AbilitySlot {
    pub tpe: AbilitySlotType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect)]
pub enum AbilitySlotType {
    WeaponAttack,
    _ShieldDefend,
}

pub struct CoreLogicPlugin;

impl Plugin for CoreLogicPlugin {
    fn build(&self, app: &mut App) {
        // from https://github.com/jakobhellermann/bevy-inspector-egui/discussions/130
        app.register_type::<Fight>()
            .register_type::<Enemy>()
            .register_type::<PlayerCharacter>();
    }
}