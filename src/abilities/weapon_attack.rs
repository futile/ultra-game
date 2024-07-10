use bevy::prelude::*;

use super::AbilityCatalog;
use crate::{
    game_logic::{
        commands::{CastAbility, CastAbilityInterface, GameCommand, GameCommandKind},
        damage_resolution::{DamageInstance, DealDamage},
        faction::Faction,
        Ability, AbilityId, AbilitySlot, AbilitySlotType,
    },
    PerUpdateSet,
};

const THIS_ABILITY_ID: AbilityId = AbilityId::Attack;

const THIS_ABILITY_DAMAGE: f64 = 51.0;

fn add_to_ability_catalog(mut abilties_catalog: ResMut<AbilityCatalog>) {
    abilties_catalog.0.insert(
        AbilityId::Attack,
        Ability {
            name: "Attack".into(),
            id: THIS_ABILITY_ID,
            slot_type: AbilitySlotType::WeaponAttack,
            description: format!("Strike with your weapon, dealing {THIS_ABILITY_DAMAGE} damage.")
                .into(),
        },
    );
}

fn cast_ability(
    mut game_commands: EventReader<GameCommand>,
    mut deal_damage_events: EventWriter<DealDamage>,
    ability_slots: Query<&AbilitySlot>,
    factions: Query<(Entity, &Faction)>,
    // ability_catalog: Res<AbilityCatalog>,
    cast_ability_interface: CastAbilityInterface,
    mut commands: Commands,
) {
    // let this_ability = ability_catalog
    //     .0
    //     .get(&THIS_ABILITY_ID)
    //     .expect("AbilityCatalog does not contain this ability");

    for cmd in game_commands.read() {
        #[expect(irrefutable_let_patterns, reason = "only one enum variant for now")]
        let GameCommand {
            source: _,
            kind:
                GameCommandKind::CastAbility(
                    cast @ CastAbility {
                        caster_e,
                        slot_e,
                        ability_e: _,
                        fight_e,
                    },
                ),
        } = cmd
        else {
            continue;
        };

        if !cast_ability_interface.is_matching_cast(cast, &THIS_ABILITY_ID) {
            continue;
        }

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

        deal_damage_events.send(DealDamage(DamageInstance {
            source: Some(*caster_e),
            target: target_e,
            amount: 51.0,
        }));

        // fire an event for the executed `GameCommand`
        commands.trigger_targets(cmd.clone(), *fight_e);
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
