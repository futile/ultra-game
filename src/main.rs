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
    _fight: Entity,
}

#[derive(Debug, Component)]
struct PlayerCharacterCard {
    _player_character: Entity,
}

#[derive(Debug, Component)]
struct EnemyCard {
    _enemy: Entity,
}

const FIGHT_BOARD_SIZE: Vec2 = Vec2::new(850., 700.);

// detect when `Fight` is added to the world
fn fight_added(
    mut commands: Commands,
    new_fights: Query<(Entity, &Fight), Added<Fight>>,
    player_characters: Query<&PlayerCharacter>,
    enemies: Query<&Enemy>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    _asset_server: Res<AssetServer>,
) {
    for (e, fight) in new_fights.iter() {
        commands
            .spawn((
                FightBoard { _fight: e },
                MaterialMesh2dBundle {
                    mesh: meshes.add(shape::Quad::new(FIGHT_BOARD_SIZE).into()).into(),
                    material: materials.add(ColorMaterial::from(Color::WHITE)),
                    transform: Transform::from_translation(Vec3::new(200., 0., 0.)),
                    ..default()
                },
            ))
            .with_children(|parent| {
                if let Ok(_pc) = player_characters.get(fight.player_character) {
                    parent
                        .spawn((
                            PlayerCharacterCard {
                                _player_character: fight.player_character,
                            },
                            SpatialBundle::from_transform(Transform::from_translation(Vec3::new(
                                -225., 225., 1.,
                            ))),
                        ))
                        .with_children(|card| {
                            card.spawn(MaterialMesh2dBundle {
                                mesh: meshes
                                    .add(shape::Quad::new(Vec2::new(350., 150.)).into())
                                    .into(),
                                material: materials
                                    .add(ColorMaterial::from(Color::BLUE.with_a(0.5))),
                                ..default()
                            });

                            card.spawn(Text2dBundle {
                                text: Text::from_section(
                                    "Player",
                                    TextStyle {
                                        font_size: 30.0,
                                        color: Color::BLACK,
                                        ..default()
                                    },
                                )
                                .with_alignment(TextAlignment::Left),
                                transform: Transform::from_translation(Vec3::new(-100., 40., 2.)),
                                ..default()
                            });
                        });
                }

                if let Ok(_enemy) = enemies.get(fight.enemy) {
                    parent
                        .spawn((
                            EnemyCard {
                                _enemy: fight.enemy,
                            },
                            SpatialBundle::from_transform(Transform::from_translation(Vec3::new(
                                225., 225., 1.,
                            ))),
                        ))
                        .with_children(|card| {
                            card.spawn(MaterialMesh2dBundle {
                                mesh: meshes
                                    .add(shape::Quad::new(Vec2::new(350., 150.)).into())
                                    .into(),
                                material: materials
                                    .add(ColorMaterial::from(Color::ORANGE_RED.with_a(0.5))),
                                ..default()
                            });

                            card.spawn(Text2dBundle {
                                text: Text::from_section(
                                    "Enemy",
                                    TextStyle {
                                        font_size: 30.0,
                                        color: Color::BLACK,
                                        ..default()
                                    },
                                )
                                .with_alignment(TextAlignment::Left),
                                transform: Transform::from_translation(Vec3::new(-100., 40., 2.)),
                                ..default()
                            });
                        });
                }
            });
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
