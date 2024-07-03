// based on
// https://gist.github.com/ItsDoot/c5e95258ec7b65fb6b2ace32fac79b7e?permalink_comment_id=4676414
// and
// 1https://github.com/bevyengine/bevy/discussions/5522#discussioncomment-9756390

use std::{
    hash::{Hash, Hasher},
    marker::PhantomData,
};

use bevy::{
    ecs::system::BoxedSystem,
    prelude::*,
    utils::{AHasher, HashMap},
};
use bevy_inspector_egui::egui::Ui;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SysId(pub u64);

impl SysId {
    pub fn new(id: impl Hash) -> Self {
        let mut hasher = AHasher::default();
        id.hash(&mut hasher);
        SysId(hasher.finish())
    }
}

#[derive(Resource)]
struct IdMappedSystems<I, O, S>
where
    I: Send + 'static,
    O: Send + 'static,
    S: Send + 'static + Sync,
{
    systems: HashMap<SysId, Option<BoxedSystem<I, O>>>,
    _phantom: PhantomData<S>,
}

impl<I, O, S> Default for IdMappedSystems<I, O, S>
where
    I: Send + 'static,
    O: Send + 'static,
    S: Send + 'static + Sync,
{
    fn default() -> Self {
        Self {
            systems: HashMap::default(),
            _phantom: PhantomData,
        }
    }
}

pub fn named_syscall<H, I, O, S, Marker>(world: &mut World, id: H, input: I, system: S) -> O
where
    H: Hash,
    I: Send + 'static,
    O: Send + 'static,
    S: IntoSystem<I, O, Marker> + Send + 'static + Sync,
{
    // the system id
    let sys_id = SysId::new(id);

    // get resource storing the id-mapped systems
    let mut id_mapped_systems =
        world.get_resource_or_insert_with::<IdMappedSystems<I, O, S>>(IdMappedSystems::default);

    // take the initialized system
    let mut system = match id_mapped_systems
        .systems
        .get_mut(&sys_id)
        .and_then(|node| node.take())
    {
        Some(system) => system,
        None => {
            let mut sys = IntoSystem::into_system(system);
            sys.initialize(world);
            Box::new(sys)
        }
    };

    // run the system
    let result = system.run(input, world);

    // apply any pending changes
    system.apply_deferred(world);

    // re-acquire mutable access to id-mapped systems
    let mut id_mapped_systems =
        world.get_resource_or_insert_with::<IdMappedSystems<I, O, S>>(IdMappedSystems::default);

    // put the system back
    // - we ignore overwrites
    match id_mapped_systems.systems.get_mut(&sys_id) {
        Some(node) => {
            let _ = node.replace(system);
        }
        None => {
            let _ = id_mapped_systems.systems.insert(sys_id, Some(system));
        }
    }

    result
}

pub fn run_ui_system<H, I, O, S, Marker>(
    ui: &mut Ui,
    world: &mut World,
    id: H,
    input: I,
    system: S,
) -> O
where
    H: Hash,
    I: Send + 'static,
    O: Send + 'static,
    S: IntoSystem<(Ui, I), (Ui, O), Marker> + Send + 'static + Sync,
{
    // create an owned child `egui::Ui` to pass to the function
    let child_ui = ui.child_ui(ui.available_rect_before_wrap(), *ui.layout());

    // then run it and grab the rendered child Ui back
    let (child_ui, output) = named_syscall(world, id, (child_ui, input), system);

    // allocate space in our parent Ui based on what was rendered inside the system
    ui.allocate_space(child_ui.min_size());

    // forward output from the system, if any
    output
}
