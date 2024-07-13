use bevy::{ecs::system::SystemParam, prelude::*};
use itertools::Itertools;

#[derive(Debug, Component, Reflect)]
pub struct HasEffects {
    // don't make this pub because there is no `OnModify`-Trigger (yet)
    holder: Entity,
}

impl HasEffects {
    pub fn new(holder: Entity) -> Self {
        Self { holder }
    }

    #[inline(always)]
    pub fn holder(&self) -> Entity {
        self.holder
    }
}

#[derive(Debug, Component, Reflect)]
pub struct EffectsHolder {
    // don't make this pub because there is no `OnModify`-Trigger (yet)
    holding_entity: Entity,
}

impl EffectsHolder {
    pub fn new(holding_entity: Entity) -> Self {
        Self { holding_entity }
    }

    #[inline(always)]
    pub fn holding_entity(&self) -> Entity {
        self.holding_entity
    }
}

#[derive(SystemParam)]
pub struct UniqueEffectInterface<'w, 's, E: Component + std::fmt::Debug> {
    has_effects: Query<'w, 's, &'static HasEffects>,
    effects_holders: Query<'w, 's, &'static EffectsHolder>,
    children: Query<'w, 's, &'static Children>,
    parents: Query<'w, 's, &'static Parent>,
    commands: Commands<'w, 's>,
    effect_query: Query<'w, 's, Entity, With<E>>,
}

impl<'w, 's, E: Component + std::fmt::Debug> UniqueEffectInterface<'w, 's, E> {
    pub fn spawn_or_replace_unique_effect(&mut self, target: Entity, effect: E) {
        let effect_e = self
            .get_unique_effect(target)
            .unwrap_or_else(|| self.spawn_effect_entity(target));

        self.commands.entity(effect_e).remove::<E>().insert(effect);
    }

    /// Removes `true` if `target` had the Effect `E` before, otherwise `false`.
    pub fn remove_unique_effect(&mut self, target: Entity) -> bool {
        if let Some(effect_e) = self.get_unique_effect(target) {
            self.commands.entity(effect_e).despawn_recursive();
            true
        } else {
            false
        }
    }

    /// Returns the Effect-`Entity` that has the component `E`, if any.
    ///
    /// `panic`s if there is more than one Effect-`Entity` with component `E`.
    pub fn get_unique_effect(&self, target: Entity) -> Option<Entity> {
        let effect_es = self.get_effects(target);
        self.effect_query
            .iter_many(effect_es)
            .at_most_one()
            .unwrap()
    }

    pub fn get_target_of_effect(&self, effect_e: Entity) -> Entity {
        let holder = self.parents.get(effect_e).unwrap().get();
        let effects_holder = self.effects_holders.get(holder).unwrap();

        effects_holder.holding_entity()
    }

    fn spawn_effect_entity(&mut self, target: Entity) -> Entity {
        let holder: Entity = match self.has_effects.get(target) {
            Ok(he) => he.holder(),
            Err(_) => {
                let holder = self.commands.spawn_empty().id();
                self.commands.entity(target).insert(HasEffects::new(holder));
                holder
            }
        };

        let new_effect = self.commands.spawn_empty().id();
        self.commands.entity(holder).add_child(new_effect);

        new_effect
    }

    fn get_effects(&self, target: Entity) -> &[Entity] {
        let Ok(holder) = self.has_effects.get(target).map(|he| he.holder()) else {
            return &[];
        };

        match self.children.get(holder) {
            Ok(children) => children,
            Err(_) => &[],
        }
    }
}

#[derive(Debug)]
pub struct EffectsPlugin;

impl Plugin for EffectsPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<HasEffects>()
            .register_type::<EffectsHolder>()
            .observe(on_add_has_effects)
            .observe(on_remove_has_effects);
    }
}

fn on_add_has_effects(
    trigger: Trigger<OnAdd, HasEffects>,
    has_effects: Query<&HasEffects>,
    effects_holder: Query<Entity, With<EffectsHolder>>,
    mut commands: Commands,
) {
    let holding_entity = trigger.entity();
    let holder = has_effects.get(holding_entity).unwrap().holder;

    // assert that the holder doesn't have `EffectsHolder` yet
    assert!(effects_holder.get(holder).is_err());

    commands
        .entity(holder)
        .insert(EffectsHolder::new(holding_entity));
}

fn on_remove_has_effects(
    trigger: Trigger<OnRemove, HasEffects>,
    has_effects: Query<&HasEffects>,
    mut commands: Commands,
) {
    let holder = has_effects.get(trigger.entity()).unwrap().holder;

    commands.entity(holder).despawn_recursive();
}
