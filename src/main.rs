use bevy::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;

#[derive(Debug, Clone, Component)]
struct Fight {
    player_character: Entity,
    enemy: Entity,
}

#[derive(Debug, Component)]
struct PlayerCharacter;

#[derive(Debug, Component)]
struct Enemy;

// detect when `Fight` is added to the world
fn fight_added(
    query: Query<
        // components
        (Entity, &Fight),
        Or<(Added<Fight>, Changed<Fight>)>,
    >,
) {
    // for (health, xp) in query.iter() {
    //     eprintln!("hp: {}+{}, xp: {}", health.hp, health.extra, xp.0);
    // }

    for (_, fight) in query.iter() {
        eprintln!("Fight added: {:?}", fight);
    }
}

fn setup(mut commands: Commands) {
    let player_character = commands
        .spawn((PlayerCharacter, Name::new("Player Character")))
        .id();
    let enemy = commands.spawn((Enemy, Name::new("The Enemy"))).id();
    commands.spawn((
        Fight {
            player_character,
            enemy,
        },
        Name::new("The Fight"),
    ));
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(WorldInspectorPlugin::new())
        .add_systems(Update, bevy::window::close_on_esc)
        .add_systems(Update, fight_added)
        .add_systems(Startup, setup)
        .run();
}
