use bevy::{ecs::system::SystemParam, prelude::*};

#[derive(Debug, Component, Reflect)]
pub struct HasEffects {
    // don't make this pub because there is no `OnModify`-Trigger (yet)
    holder: Entity,
}

impl HasEffects {
    pub fn new(holder: Entity) -> Self {
        Self { holder }
    }

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

    pub fn holding_entity(&self) -> Entity {
        self.holding_entity
    }
}

#[derive(SystemParam)]
pub struct EffectsInterface<'w, 's> {
    has_effects: Query<'w, 's, &'static HasEffects>,
    children: Query<'w, 's, &'static Children>,
    commands: Commands<'w, 's>,
}

impl<'w, 's> EffectsInterface<'w, 's> {
    pub fn spawn_effect(&mut self, target: Entity) -> Entity {
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

    pub fn get_effects(&self, target: Entity) -> &[Entity] {
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
