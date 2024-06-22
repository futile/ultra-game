use bevy::prelude::*;

use super::AbilityCatalog;
use crate::{
    game_logic::{
        commands,
        damage_resolution::{DamageInstance, DealDamage},
        faction::Faction,
        Ability, AbilityId, AbilitySlot, AbilitySlotType,
    },
    PerUpdateSet,
};

const THIS_ABILITY_ID: AbilityId = AbilityId::Attack;

fn add_to_ability_catalog(mut abilties_catalog: ResMut<AbilityCatalog>) {
    abilties_catalog.0.insert(
        AbilityId::Attack,
        Ability {
            name: "Attack".into(),
            id: THIS_ABILITY_ID,
            slot_type: AbilitySlotType::WeaponAttack,
        },
    );
}

fn cast_ability(
    mut cast_ability_events: EventReader<commands::CastAbility>,
    mut deal_damage_events: EventWriter<DealDamage>,
    ability_ids: Query<&AbilityId>,
    ability_slots: Query<&AbilitySlot>,
    factions: Query<(Entity, &Faction)>,
    ability_catalog: Res<AbilityCatalog>,
) {
    let this_ability = ability_catalog
        .0
        .get(&THIS_ABILITY_ID)
        .expect("AbilityCatalog does not contain this ability");

    for commands::CastAbility {
        caster_e,
        slot_e,
        ability_e: _,
        fight_e,
    } in cast_ability_events
        .read()
        .filter(|c| c.is_valid_matching_ability_cast(this_ability, &ability_ids, &ability_slots))
    {
        let slot: Option<&AbilitySlot> = slot_e.map(|slot_e| ability_slots.get(slot_e).unwrap());
        let (_, faction) = factions.get(*caster_e).unwrap();

        let (target_e, _target_faction) = faction.find_single_enemy(&factions);

        println!(
            "Casting ability: {THIS_ABILITY_ID:?} | Fight: {fight_e:?} | Caster: {caster_e:?} | Slot: {slot_e:?} [{slot:?}] | Target: {target_e:?}"
        );

        deal_damage_events.send(DealDamage(DamageInstance {
            source: Some(*caster_e),
            target: target_e,
            amount: 13.37,
        }));
    }
}

#[derive(Debug)]
pub struct WeaponAttackPlugin;

impl Plugin for WeaponAttackPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, add_to_ability_catalog)
            .add_systems(Update, cast_ability.in_set(PerUpdateSet::CommandResolution));
    }
}
