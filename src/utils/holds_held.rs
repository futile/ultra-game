use std::marker::PhantomData;

use bevy::prelude::*;

#[derive(Debug, Component, Reflect)]
#[relationship(relationship_target = Holds<T>)]
pub struct Held<T: Send + Sync + 'static> {
    #[relationship]
    pub held_by: Entity,
    #[reflect(ignore)]
    pub _phantom_t: PhantomData<T>,
}

#[derive(Debug, Component, Reflect)]
#[relationship_target(relationship = Held<T>,  linked_spawn)]
pub struct Holds<T: Send + Sync + 'static> {
    #[relationship]
    held_entities: Vec<Entity>,
    #[reflect(ignore)]
    _phantom_t: PhantomData<T>,
}
