use std::{borrow::Cow, sync::OnceLock};

use bevy::{
    prelude::*,
    utils::{Duration, HashMap},
};

use crate::core_logic::AbilitySlotType;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AbilityId {
    Attack,
}

#[derive(Debug)]
pub struct Ability {
    name: Cow<'static, str>,
    id: AbilityId,
    slot_type: AbilitySlotType,
    cooldown: Duration,
}

#[derive(Debug, Resource, Reflect, Default)]
#[reflect(Resource)]
pub struct AbilityCatalog;

impl std::ops::Deref for AbilityCatalog {
    type Target = HashMap<AbilityId, Ability>;
    fn deref(&self) -> &Self::Target {
        ability_catalog()
    }
}

pub fn ability_catalog() -> &'static HashMap<AbilityId, Ability> {
    static ABILITY_CATALOG: OnceLock<HashMap<AbilityId, Ability>> = OnceLock::new();
    ABILITY_CATALOG.get_or_init(|| {
        let mut catalog = HashMap::new();
        catalog.insert(
            AbilityId::Attack,
            Ability {
                name: "Attack".into(),
                id: AbilityId::Attack,
                slot_type: AbilitySlotType::WeaponAttack,
                cooldown: Duration::from_secs_f32(1.0),
            },
        );
        catalog
    })
}

#[derive(Debug)]
pub struct AbilityCatalogPlugin;

impl Plugin for AbilityCatalogPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(AbilityCatalog)
            .register_type::<AbilityCatalog>();
    }
}
