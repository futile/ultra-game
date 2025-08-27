use bevy::{ecs::system::SystemParam, prelude::*};
use derive_more::From;

use crate::{
    abilities::AbilityInterface,
    game_logic::{
        ability::AbilityId,
        ability_slots::AbilitySlot,
        fight::{FightInterface, FightStatus},
    },
};

#[derive(Debug, Clone, Event)]
pub struct GameCommand {
    pub source: GameCommandSource,
    pub kind: GameCommandKind,
}

impl GameCommand {
    pub fn new(source: GameCommandSource, kind: GameCommandKind) -> Self {
        Self { source, kind }
    }

    pub fn new_from_user(kind: GameCommandKind) -> Self {
        Self::new(GameCommandSource::UserInteraction, kind)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GameCommandSource {
    UserInteraction,
}

#[derive(Debug, Clone, From)]
pub enum GameCommandKind {
    UseAbility(UseAbility),
}

#[derive(Debug, Clone)]
pub struct UseAbility {
    pub caster_e: Entity,
    pub slot_e: Entity,
    pub ability_e: Entity,
    pub fight_e: Entity,
}

#[derive(SystemParam)]
pub struct CastAbilityInterface<'w, 's> {
    ability_ids: Query<'w, 's, &'static AbilityId>,
    ability_slots: Query<'w, 's, &'static AbilitySlot>,
    ability_interface: AbilityInterface<'w, 's>,
    fight_interface: FightInterface<'w, 's>,
}

impl<'w, 's> CastAbilityInterface<'w, 's> {
    pub fn is_matching_cast(&self, cast: &UseAbility, id: &AbilityId) -> bool {
        let ability_id = self.ability_ids.get(cast.ability_e).unwrap();
        ability_id == id
    }

    pub fn is_valid_cast(&self, cast: &UseAbility) -> bool {
        match self.fight_interface.get_fight_status(cast.fight_e) {
            FightStatus::Ongoing => (),
            FightStatus::Ended => return false,
        };

        let ability = self
            .ability_interface
            .get_ability_from_entity(cast.ability_e);
        let slot = self.ability_slots.get(cast.slot_e).unwrap();

        let can_use_slot = ability.can_use_slot(slot);

        #[expect(clippy::let_and_return, reason = "better readability")]
        can_use_slot
    }
}

pub struct CommandsPlugin;

impl Plugin for CommandsPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<GameCommand>();
    }
}
