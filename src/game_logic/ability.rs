use std::borrow::Cow;

use bevy::prelude::*;

use crate::{
    game_logic::ability_slots::{AbilitySlot, AbilitySlotType},
    utils::holds_held::{Held, Holds},
};

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
    pub description: Cow<'static, str>,
}

impl Ability {
    pub fn can_use_slot(&self, selected_ability_slot: &AbilitySlot) -> bool {
        let selected_slot_type = Some(selected_ability_slot.tpe);

        self.slot_type == selected_slot_type
    }
}

pub struct AbilityPlugin;

impl Plugin for AbilityPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Holds<Ability>>()
            .register_type::<Held<Ability>>()
            .register_type::<AbilityId>();
    }
}
