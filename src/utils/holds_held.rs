use std::marker::PhantomData;

use bevy::prelude::*;

#[derive(Component)]
#[relationship(relationship_target = Holds<T>)]
pub struct Held<T: Send + Sync + 'static> {
    #[relationship]
    pub held_by: Entity,
    _phantom_t: PhantomData<T>,
}

#[derive(Component)]
#[relationship_target(relationship = Held<T>,  linked_spawn)]
pub struct Holds<T: Send + Sync + 'static> {
    #[relationship]
    held_entities: Vec<Entity>,
    _phantom_t: PhantomData<T>,
}
