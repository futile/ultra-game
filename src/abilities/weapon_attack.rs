use bevy::prelude::*;

use super::AbilityCatalog;
use crate::{
    game_logic::{
        commands::{self, CastAbilityInterface},
        damage_resolution::{DamageInstance, DealDamage},
        faction::Faction,
        fight::{Fight, FightTime},
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
    ability_slots: Query<&AbilitySlot>,
    factions: Query<(Entity, &Faction)>,
    // ability_catalog: Res<AbilityCatalog>,
    mut fight_times: Query<&mut FightTime, With<Fight>>,
    cast_ability_interface: CastAbilityInterface,
) {
    // let this_ability = ability_catalog
    //     .0
    //     .get(&THIS_ABILITY_ID)
    //     .expect("AbilityCatalog does not contain this ability");

    for cast @ commands::CastAbility {
        caster_e,
        slot_e,
        ability_e: _,
        fight_e,
    } in cast_ability_events
        .read()
        .filter(|cast| cast_ability_interface.is_matching_cast(cast, &THIS_ABILITY_ID))
    {
        if !cast_ability_interface.is_valid_cast(cast) {
            warn!("invalid `CastAbility`: {cast:#?}");
            continue;
        }

        let slot: Option<&AbilitySlot> = slot_e.map(|slot_e| ability_slots.get(slot_e).unwrap());
        let (_, faction) = factions.get(*caster_e).unwrap();

        let (target_e, _target_faction) = faction.find_single_enemy(&factions);

        println!(
            "Casting ability: {THIS_ABILITY_ID:?} | Fight: {fight_e:?} | Caster: {caster_e:?} | Slot: {slot_e:?} [{slot:?}] | Target: {target_e:?}"
        );

        // unpause fight if it was paused.
        // TODO: should not happen here, should happen if *any* (player?) command was accepted!
        fight_times.get_mut(*fight_e).unwrap().stop_watch.unpause();

        deal_damage_events.send(DealDamage(DamageInstance {
            source: Some(*caster_e),
            target: target_e,
            amount: 51.0,
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
