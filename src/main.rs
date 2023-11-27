use bevy::{prelude::*, sprite::MaterialMesh2dBundle};
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

#[derive(Debug, Component)]
struct FightBoard {
    fight: Entity,
}

// detect when `Fight` is added to the world
fn fight_added(
    mut commands: Commands,
    query: Query<Entity, Added<Fight>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for e in query.iter() {
        println!("Detected new `Fight`, spawning `FightBoard`");

        commands.spawn((
            FightBoard { fight: e },
            MaterialMesh2dBundle {
                mesh: meshes
                    .add(shape::Quad::new(Vec2::new(850., 700.)).into())
                    .into(),
                material: materials.add(ColorMaterial::from(Color::GRAY)),
                transform: Transform::from_translation(Vec3::new(200., 0., 0.)),
                ..default()
            },
        ));
    }
}

fn fight_changed(query: Query<Entity, Changed<Fight>>) {
    for _ in query.iter() {
        println!("Detected changed `Fight`");
    }
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());

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
        .add_systems(Update, fight_changed)
        .add_systems(Startup, setup)
        .run();
}
