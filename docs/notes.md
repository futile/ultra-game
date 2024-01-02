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
