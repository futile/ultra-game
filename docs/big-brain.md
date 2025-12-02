# big-brain AI Integration Guide

This document provides a Claude-friendly guide to using the `big-brain` crate for AI behaviors in our Bevy-based combat game.

## Overview

`big-brain` is a Utility AI library designed specifically for Bevy that enables data-driven AI decision-making through modular, composable components. It uses a concurrent evaluation system that fits perfectly with Bevy's ECS architecture.

**Version**: 0.22 (compatible with Bevy 0.16.1)

## Core Concepts

### 1. Scorers
Scorers are the "eyes" of the AI system - they evaluate the game world and produce `Score` values (0.0-1.0 range).

```rust
#[derive(Component, Debug, ScorerBuilder)]
pub struct HealthScorer;

pub fn health_scorer_system(
    mut scorers: Query<(&Actor, &mut Score), With<HealthScorer>>,
    actors: Query<&Health>,
) {
    for (Actor(actor), mut score) in &mut scorers {
        if let Ok(health) = actors.get(*actor) {
            score.set(health.current / health.max);
        }
    }
}
```

### 2. Actions
Actions define specific behaviors that entities can perform. They have states: `Requested`, `Executing`, `Success`, or `Failure`.

```rust
#[derive(Component, Debug, ActionBuilder)]
pub struct AttackAction;

pub fn attack_action_system(
    mut actions: Query<(&Actor, &mut ActionState), With<AttackAction>>,
    // your game logic queries here
) {
    for (Actor(actor), mut action_state) in &mut actions {
        match *action_state {
            ActionState::Requested => {
                // Start the attack
                *action_state = ActionState::Executing;
            }
            ActionState::Executing => {
                // Continue or finish the attack
                *action_state = ActionState::Success;
            }
            _ => {}
        }
    }
}
```

### 3. Thinkers
Thinkers are the central "brain" component that connects Scorers and Actions. They use Pickers to decide which Action to execute.

```rust
use big_brain::prelude::*;

// Create a Thinker for an enemy entity
commands.entity(enemy_entity).insert(
    Thinker::build()
        .picker(FirstToScore { threshold: 0.8 })
        .when(HealthScorer, HealAction)
        .when(EnemyNearbyScorer, AttackAction)
        .when(NoTargetScorer, PatrolAction)
);
```

## Integration with Ultra-Game Architecture

### Setup with Your Plugin System

Add the BigBrain plugin and your AI systems:

```rust
use big_brain::BigBrainPlugin;
use crate::game_logic::PerUpdateSet;

fn main() {
    App::new()
        .add_plugins(BigBrainPlugin::new(PreUpdate))
        .add_systems(PreUpdate, (
            health_scorer_system,
            enemy_nearby_scorer_system,
            attack_action_system,
            heal_action_system,
        ).in_set(PerUpdateSet::LogicUpdate))
        .run();
}
```

### Combat-Specific AI Patterns

#### Target Selection Scorer
```rust
#[derive(Component, Debug, ScorerBuilder)]
pub struct EnemyInRangeScorer {
    pub range: f32,
}

pub fn enemy_in_range_scorer_system(
    mut scorers: Query<(&Actor, &mut Score, &EnemyInRangeScorer)>,
    positions: Query<&Transform>,
    enemies: Query<Entity, (With<Health>, Without<Ally>)>,
) {
    for (Actor(actor), mut score, range_scorer) in &mut scorers {
        let Ok(actor_pos) = positions.get(*actor) else { continue };
        
        let closest_enemy_distance = enemies.iter()
            .filter_map(|enemy| positions.get(enemy).ok())
            .map(|enemy_pos| actor_pos.translation.distance(enemy_pos.translation))
            .min_by(|a, b| a.partial_cmp(b).unwrap());
            
        if let Some(distance) = closest_enemy_distance {
            let normalized_score = (range_scorer.range - distance).max(0.0) / range_scorer.range;
            score.set(normalized_score.clamp(0.0, 1.0));
        } else {
            score.set(0.0);
        }
    }
}
```

#### Ability Usage Action
This integrates with your existing ability system:

```rust
#[derive(Component, Debug, ActionBuilder)]
pub struct UseAbilityAction {
    pub ability_entity: Entity,
    pub slot_entity: Entity,
    pub target: Option<Entity>,
}

pub fn use_ability_action_system(
    mut actions: Query<(&Actor, &mut ActionState, &UseAbilityAction)>,
    mut ability_interface: AbilityCastingInterface,
    mut commands: MessageWriter<GameCommand>,
) {
    for (Actor(actor), mut action_state, ability_action) in &mut actions {
        match *action_state {
            ActionState::Requested => {
                // Create UseAbility command
                let use_ability = UseAbility {
                    caster_e: *actor,
                    slot_e: ability_action.slot_entity,
                    ability_e: ability_action.ability_entity,
                    target: ability_action.target,
                    fight_e: /* get fight entity */,
                };
                
                // Check if we can cast the ability
                if ability_interface.is_valid_cast(&use_ability).is_ok() {
                    // Send command - validation and execution handled automatically
                    commands.write(GameCommand {
                        kind: GameCommandKind::UseAbility(use_ability),
                    });
                    *action_state = ActionState::Success;
                } else {
                    *action_state = ActionState::Failure;
                }
            }
            _ => {}
        }
    }
}
```

### Working with Your Entity Relationships

Big-brain works well with your `Holds<T>/Held<T>` pattern:

```rust
// Scorer that checks if an entity holds a weapon
#[derive(Component, Debug, ScorerBuilder)]
pub struct HasWeaponScorer;

pub fn has_weapon_scorer_system(
    mut scorers: Query<(&Actor, &mut Score), With<HasWeaponScorer>>,
    weapon_holders: Query<&Holds<Weapon>>,
) {
    for (Actor(actor), mut score) in &mut scorers {
        if weapon_holders.contains(*actor) {
            score.set(1.0);
        } else {
            score.set(0.0);
        }
    }
}
```

### Cooldown-Aware AI

Integrate with your cooldown system:

```rust
#[derive(Component, Debug, ScorerBuilder)]
pub struct AbilityCooldownScorer {
    pub ability_entity: Entity,
}

pub fn ability_cooldown_scorer_system(
    mut scorers: Query<(&Actor, &mut Score, &AbilityCooldownScorer)>,
    cooldowns: Query<Has<Cooldown>>,
) {
    for (Actor(actor), mut score, cooldown_scorer) in &mut scorers {
        // Check if ability has cooldown component
        if cooldowns.get(cooldown_scorer.ability_entity).unwrap_or(false) {
            score.set(0.0); // Ability is on cooldown
        } else {
            score.set(1.0); // Ability is ready
        }
    }
}
```

## Common AI Behaviors

### Basic Enemy AI Template

```rust
pub fn spawn_basic_enemy(
    commands: &mut Commands,
    position: Transform,
    weapon_attack_ability: Entity,
    weapon_attack_slot: Entity,
) -> Entity {
    let enemy = commands.spawn((
        // Your existing enemy components
        Health::new(100.0),
        position,
        // AI components
        Thinker::build()
            .picker(Highest)
            .when(
                EnemyInRangeScorer { range: 2.0 },
                UseAbilityAction {
                    ability_entity: weapon_attack_ability,
                    slot_entity: weapon_attack_slot,
                    target: None, // Target will be determined in action system
                }
            )
            .when(
                EnemyNearbyScorer { range: 10.0 },
                MoveTowardsEnemyAction
            )
            .when(
                NoEnemyNearbyScorer { range: 15.0 },
                PatrolAction
            )
    )).id();

    enemy
}
```

### Advanced Target Selection

```rust
#[derive(Component, Debug, ScorerBuilder)]
pub struct BestTargetScorer;

pub fn best_target_scorer_system(
    mut scorers: Query<(&Actor, &mut Score), With<BestTargetScorer>>,
    positions: Query<&Transform>,
    healths: Query<&Health>,
    enemies: Query<Entity, (With<Health>, Without<Ally>)>,
) {
    for (Actor(actor), mut score) in &mut scorers {
        let Ok(actor_pos) = positions.get(*actor) else { continue };
        
        let best_target_score = enemies.iter()
            .filter_map(|enemy| {
                let enemy_pos = positions.get(enemy).ok()?;
                let enemy_health = healths.get(enemy).ok()?;
                let distance = actor_pos.translation.distance(enemy_pos.translation);
                
                // Score based on: low health (easier to kill) and close distance
                let health_score = 1.0 - (enemy_health.current / enemy_health.max);
                let distance_score = (10.0 - distance).max(0.0) / 10.0;
                Some((health_score + distance_score) / 2.0)
            })
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(0.0);
            
        score.set(best_target_score);
    }
}
```

## Debugging and Development

### Using Reflection for AI State Inspection

Big-brain supports reflection, which works well with bevy-inspector-egui:

```rust
#[derive(Component, Debug, ScorerBuilder, Reflect)]
#[reflect(Component)]
pub struct CustomScorer {
    pub some_parameter: f32,
}
```

### System Ordering

Ensure AI systems run in the correct order within your `PerUpdateSet`:

```rust
.add_systems(PreUpdate, (
    // Scorers first
    (health_scorer_system, enemy_nearby_scorer_system),
    // Then actions (big-brain handles the Thinker logic automatically)
    (attack_action_system, heal_action_system),
).chain().in_set(PerUpdateSet::LogicUpdate))
```

## Performance Considerations

- Scorers run in parallel by default - leverage this for performance
- Use `Schedule::run_if()` to conditionally run AI systems (e.g., only when game is not paused)
- Consider using time-based AI evaluation rather than every frame for less critical decisions
- Big-brain integrates well with your existing pause system since it respects system scheduling

## Key Takeaways

1. **Modular Design**: Create reusable Scorers and Actions that can be mixed and matched
2. **Integration**: Big-brain works seamlessly with your existing ECS architecture
3. **Combat Focus**: Use Scorers for target selection, threat assessment, and ability readiness
4. **State Management**: Actions integrate naturally with your command pattern and ability system
5. **Debugging**: Leverage reflection and Bevy's inspector for AI state visibility

This setup gives you a solid foundation for implementing sophisticated enemy AI that respects your game's combat rules and systems.

## References

- **Official Documentation**: https://docs.rs/big-brain/latest/big_brain/index.html
- **GitHub Repository**: https://github.com/zkat/big-brain
- **Example Code**: https://raw.githubusercontent.com/zkat/big-brain/main/examples/thirst.rs