use std::time::Duration;

use bevy::prelude::*;

use super::AbilityCatalog;
use crate::{
    PerUpdateSet,
    game_logic::{
        ability::{Ability, AbilityId},
        ability_casting::{AbilityCastingInterface, UseAbility},
        ability_slots::{AbilitySlot, AbilitySlotType},
        commands::{GameCommand, GameCommandKind},
        cooldown::Cooldown,
        damage_resolution::{DamageInstance, DealDamage},
        faction::Faction,
    },
};

const THIS_ABILITY_ID: AbilityId = AbilityId::Attack;
const THIS_ABILITY_DAMAGE: f64 = 10.0;
const THIS_ABILITY_ABILITY_COOLDOWN: Duration = Duration::from_secs(5);

fn add_to_ability_catalog(mut abilties_catalog: ResMut<AbilityCatalog>) {
    abilties_catalog.0.insert(
        THIS_ABILITY_ID,
        Ability {
            name: "Attack".into(),
            id: THIS_ABILITY_ID,
            slot_type: Some(AbilitySlotType::WeaponAttack),
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
    mut ability_casting_interface: AbilityCastingInterface,
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
                GameCommandKind::UseAbility(
                    cast @ UseAbility {
                        caster_e,
                        slot_e,
                        ability_e,
                        fight_e,
                    },
                ),
        } = cmd
        else {
            continue;
        };

        if !ability_casting_interface.is_matching_cast(cast, &THIS_ABILITY_ID) {
            continue;
        }

        if let Err(e) = ability_casting_interface.is_valid_cast(cast) {
            warn!("invalid `CastAbility`: {cast:#?}, reason: {e}");
            continue;
        }

        let slot = ability_slots.get(*slot_e).unwrap();
        let (_, faction) = factions.get(*caster_e).unwrap();

        let (target_e, _target_faction) = faction.find_single_enemy(&factions);

        println!(
            "Casting ability: {THIS_ABILITY_ID:?} | Fight: {fight_e:?} | Caster: {caster_e:?} | Slot: {slot_e:?} [{slot:?}] | Target: {target_e:?}"
        );

        // use the slot (so, e.g., ongoing casts can be interrupted)
        ability_casting_interface.use_slot(*slot_e);

        // start cooldown on the ability
        commands
            .entity(*ability_e)
            .insert(Cooldown::new(THIS_ABILITY_ABILITY_COOLDOWN));

        // trigger/send damage event
        deal_damage_events.write(DealDamage(DamageInstance {
            source: Some(*caster_e),
            target: target_e,
            amount: THIS_ABILITY_DAMAGE,
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
