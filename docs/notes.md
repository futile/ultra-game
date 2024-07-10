# Todos

## Next:
- [ ] Figure out what's next :) More complicated abilities? More units per side? Buffs/effects/hexes/other name? AI :O?

## Possible Next
- [ ] 1 or 2 basic tests, also just to try out how well it works/how easy it is

- [ ] Proper Fight-Over Tracking and Handling
  - [x] Command Submission + Execution should be disabled (maybe also print/log) a warning
    - [ ] In the Systems: Have an `In<EntityList>` or sth. parameter, and another (generic?) system that gathers
          active Fights and runs the System for each one's Entities.
      - [ ] How to make Events per-Fight? Necessary?
        - Use `Observer`s on the `Fight` entity, and trigger events on it!
      - This could use bevy's `SubApp`s, but they are pretty much tied to rendering.
      - Subworlds are a feature that is planned in bevy, but not being worked anytime soon-ish it seems: https://github.com/bevyengine/rfcs/pull/16
      - Only `(Fixed-)Update` systems or also event-based systems?
      - -> All of this: Later

- Targeting-system for abilities (UI)
    - Targeting-state for UI when selecting an ability
    - Cancel using ESC?
    - Re-use number-hotkeys?
    - Targetable enemies/entities/units have to be tracked through castability-system (see below).
    - Add `target` entity/unit to `commands::CastAbility`, maybe add optional targeting or new command type.

---
# Done
---
- [x] Tooltips for Abilities (/Slots? probably later, start with abilities)

- [x] Event when a `command` is accepted with trigger
  - [x] Use `command`-accepted event to unpause fight_time (if paused)

- [x] Wrapper-Type for the different `game_logic::commands` that we can have, with tracking where it's from
  - I.e., `UserInteraction`, `AIDecision`, etc. (very slightly thinking about Replays, but not really atm)

- [x] Update to bevy 0.14
  - Tried 2024-07-05 (bevy 0.14 just released); `bevy_egui{,_inspector}` not yet upgradeable. Rest looks ok.
    - Use `just cargo-update-breaking` for trying again

- [x] Feat to "pause" a fight, until resumed/a Command is given
  - Pause key `<space>` for the start at least

- [x] Track & show fight stopwatch time

- [x] Fight over tracking: UI should also be disabled (implicitly, maybe also explicitly?)
  - [x] Abilities-/Command-sections are implicitly disabled (through, e.g., checking `CastAblity::is_valid_cast()`)
  - [x] Slots-section needs to be explicitly disabled (at least for now)

- [x] `Ongoing` or `Finished` only for now (new Component(s), check for `FightResult`?)

- [x] UI should show a `"'{Faction}' won!"`-headline or sth. when the Fight is over

- [x] Alternative: Implement damage, hp, etc. first, and simply make `Attack` target all/the first/etc. enemy.
    - Also ok, maybe much easier, but probably better, because it forces me to work on actual gameplay, instead of building systems :see_no_evil:

# Notes

## Castability etc. systems/handling

Casting an ability has different kinds of information that determine castability.

1. Is the ability itself ready *locally*, i.e., cooldown and manacost ready?
    - Determine this through a system/systems and keep it updated on the ability in some way.
    - E.g., a component, etc.
    - Static information is pulled from the ability definition, and can be overriden dynamically through components.
      E.g.: A "Disabled for 10 sec"-component would cause the ability to not be ready, even though static stuff such as cooldown and manacost are ready.
2. Which slots can it be used with?
    - Keep these as (aery?) edges, i.e., ability is connected to all slots it can be used with.
    - If an ability requires multiple slots, create a "hyper slot", which represents a combination of two (or more) slots. Connect with this then.
3. Which targets can the ability be used on?
    - Keep these as (aery?) edges, and keep it up to date.
    - Similar to slots, static + dynamic information/components can (probably) be involved.
4. Is the caster itself able to cast an ability (this ability?) at all?
    - E.g., if silenced/muted or something, can't cast.

All of these can be resolved/kept up to date individually.
Finally, executability of a command, such as `CastAbility`, checks all of them to determine castability.

---
# OLD STUFF
---

# Idea for Ability Casting / General Entity-Command System

Dual-phase for actions:

1. Register which data, e.g., ability usability, is requested.
    - No concrete plan on this yet, probably a bevy `Event`?
    - Needs a back-channel somehow, so might make sense to store it on an entity/component somehow?
2. System(s) run that resolve the request, e.g., can the entity currently cast the ability or not?
    - General system? Or per ability? Both make sense, maybe per ability is nice, keeps the logic contained.
    - Shared functionality should be easy through functions etc.
3. System that wants to use the data runs again with the result of the request.
    - Can now do whatever it needed the data for.

## Example: UI: Which abilities can be cast with the selected Ability Slot?

1. UI's "pre"-/"request"-phase runs.
    1. It submits requests for each registered ability and the selected ability slot (or no slot).
2. Abilities' general/per-ability systems run and resolve the requests, storing the result somewhere known to the caller.
3. UI's "main"-phase runs, in which it retrieves the results of its requests and uses them to render the ui, enabling/disabling ability buttons.
