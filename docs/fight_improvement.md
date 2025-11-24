# Fight Improvement Analysis

## Current Fight Stats

### Health Values

| Entity | Health |
| ------ | ------ |
| Player | 100.0  |
| Enemy  | 100.0  |

### Ability Details

Abilities are now entities with components defining their properties:

| Ability           | Slot Type    | Damage     | Cast Time | Ability Cooldown | Slot Cooldown | Notes                            |
| ----------------- | ------------ | ---------- | --------- | ---------------- | ------------- | -------------------------------- |
| **WeaponAttack**  | WeaponAttack | 10.0       | Instant   | 5s               | 1s            | Basic attack                     |
| **ChargedStrike** | WeaponAttack | 25.0       | 2s cast   | 20s              | 1s            | High damage, long cooldown       |
| **NeedlingHex**   | Magic        | 25.0 total | Instant   | 30s              | 2s            | DoT: 5.0 dmg × 5 ticks over 2.5s |

- Ability properties are defined via components: `AbilityCooldown`, `AbilityCastTime`, `AbilitySlotRequirement`
- All abilities go through `OngoingCast` → `PerformAbility` flow (instant abilities have Duration::ZERO cast time)
- Specific ability logic is implemented as observers responding to `PerformAbility` events

### Detailed NeedlingHex Mechanics

- **Tick Interval**: 500ms (0.5 seconds)
- **Number of Ticks**: 5
- **Damage per Tick**: 5.0
- **Total Duration**: 2.5 seconds
- **Total Damage**: 25.0

### Cooldown Mechanics Clarification

**Important distinction between ability and slot cooldowns:**

- **Ability Cooldown**: How long until THIS specific ability can be cast again
- **Slot Cooldown**: How long until ANY ability can be cast on this slot

**Example**: After casting ChargedStrike (20s ability, 1s slot cooldown):

- WeaponAttack slot is blocked for 1s
- At t=3s, WeaponAttack becomes available (5s ability cooldown < elapsed time)
- ChargedStrike itself can't be cast again until t=22s

This means WeaponAttack becomes available much sooner than initially calculated.

## Current Fight Flow Analysis

### Opening Sequence (Player)

1. **ChargedStrike** (25 dmg) → 2s cast → 20s ability cooldown, 1s slot cooldown
2. **NeedlingHex** (25 dmg over 2.5s) → 30s ability cooldown, 2s slot cooldown
3. **WeaponAttack** available at t=3s (when WeaponAttack slot cooldown expires)

### Problem: Still Somewhat Boring Mid-Fight

After the opening burst (t=0-3s), the player has:

- ChargedStrike: 17s cooldown remaining (available at t=22s)
- NeedlingHex: 27s cooldown remaining (available at t=32s)
- WeaponAttack: Available every 5s starting at t=3s

The issue is reduced but still exists: WeaponAttack every 5s for the next ~20 seconds until ChargedStrike returns.

## Potential Improvements

### 1. More Abilities

- Add additional abilities with shorter cooldowns
- Provide more tactical choices during downtime

### 2. Resource Management

- Add mana/stamina system to create resource trade-offs
- Abilities cost resources, forcing strategic timing

### 3. Cooldown Adjustments

- Reduce ability cooldowns to increase action frequency
- Adjust damage to maintain balance

### 4. Reactive Mechanics

- Add abilities that trigger on enemy actions
- Defensive abilities or counters

### 5. Combo System

- Abilities that synergize with each other
- Sequential ability bonuses

### 6. Environmental Factors

- Positioning mechanics
- Temporary buffs/debuffs that change tactics

## Proposed Reactive Mechanic: Enemy ChargedStrike + Player Block

### New Abilities

| Ability           | Entity | Slot Type    | Damage/Effect      | Cast Time  | Cooldown | Trigger      |
| ----------------- | ------ | ------------ | ------------------ | ---------- | -------- | ------------ |
| **ChargedStrike** | Enemy  | WeaponAttack | 25.0               | 2s cast    | 20s      | At 35 health |
| **Block**         | Player | ShieldDefend | Blocks next damage | 1s channel | 10s      | Manual cast  |

### Block Mechanics

- **Channel Duration**: 1 second
- **Effect**: Blocks the next damage instance received during the 1s channel
- **Slot**: ShieldDefend (currently unused)
- **Strategic Use**: Must be timed to block enemy's ChargedStrike

### Fight Simulation with New Mechanics

**Phase 1: Opening Burst (Enemy 100 → 35 health)**

- t=0-2s: Player casts ChargedStrike (enemy attacks once for 10 dmg, player at 90)
- t=2s: ChargedStrike hits → Enemy at 75 health
- t=2s: Player casts NeedlingHex (instant) → Enemy starts taking DoT
- t=2.5-4.5s: NeedlingHex deals 25 damage over 2.5s → Enemy at 50 health
- t=3s: WeaponAttack slot becomes available (1s slot cooldown expired)
- t=3s: Player uses WeaponAttack → Enemy at 40 health
- t=4s: Player uses WeaponAttack → Enemy at 30 health (crosses 35 threshold)
- Enemy attacks 2-3 more times during t=2-4s → Player at 70-80 health

**Phase 2: Enemy ChargedStrike Response**

- t=4s: Enemy health drops below 35, **triggers ChargedStrike cast** (2s cast time)
- t=4-6s: **Critical decision window** - Player has 2s to react

**Player Options:**

1. **Aggressive**: Continue attacking, take 25 damage (drop to 45 health)
2. **Defensive**: Cast Block at t=5s (1s channel), negate the ChargedStrike damage

**Phase 3: Resolution**

- t=6s: Enemy's ChargedStrike completes
  - If blocked: No damage, player still at 70-80 health
  - If not blocked: 25 damage, player at 45-55 health
- t=6s+: Continue with WeaponAttack every 5s until enemy dies (at ~t=8-10s)

### Analysis: Still Not Engaging Enough

**Problems with this approach:**

1. **Timing too predictable**: Enemy always casts at 35 health
2. **Binary decision**: Block or don't block - not much strategy
3. **One-time event**: After the ChargedStrike, back to WeaponAttack spam
4. **Low stakes**: Player can survive either choice easily

**The fundamental issue remains**: This adds one tactical moment but doesn't solve the core problem of long periods with only WeaponAttack available.

**Better alternatives might include:**

- Multiple enemy abilities with different timings
- Player abilities with shorter cooldowns for sustained engagement
- Resource management that creates ongoing decision-making
- Combo systems that reward skillful ability sequencing

=> Players needs something like a "rotation" of abilities to use (constantly), plus abilities to dodge things, etc., and necessary timings to use these.
