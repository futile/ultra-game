use bevy::{prelude::*, sprite::MaterialMesh2dBundle};

use super::ability_slots_section::AbilitySlotsSection;
use crate::core_logic::{Enemy, Fight, PlayerCharacter};

// detect when `Fight` is added to the world
pub fn fight_added(
    mut commands: Commands,
    new_fights: Query<(Entity, &Fight), Added<Fight>>,
    player_characters: Query<&PlayerCharacter>,
    enemies: Query<&Enemy>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
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
                                -225., 0., 1.,
                            ))),
                        ))
                        .with_children(|card| {
                            card.spawn(MaterialMesh2dBundle {
                                mesh: meshes.add(shape::Quad::new(CARD_SIZE).into()).into(),
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
                                transform: CARD_TEXT_TRANSFORM,
                                ..default()
                            });

                            card.spawn((
                                AbilitySlotsSection::new(fight.player_character),
                                SpatialBundle::from_transform(Transform::from_translation(
                                    Vec3::new(-50., 150., 1.),
                                )),
                            ));
                        });
                }

                if let Ok(_enemy) = enemies.get(fight.enemy) {
                    parent
                        .spawn((
                            EnemyCard {
                                _enemy: fight.enemy,
                            },
                            SpatialBundle::from_transform(Transform::from_translation(Vec3::new(
                                225., 0., 1.,
                            ))),
                        ))
                        .with_children(|card| {
                            card.spawn(MaterialMesh2dBundle {
                                mesh: meshes.add(shape::Quad::new(CARD_SIZE).into()).into(),
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
                                transform: CARD_TEXT_TRANSFORM,
                                ..default()
                            });

                            card.spawn((
                                AbilitySlotsSection::new(fight.enemy),
                                SpatialBundle::from_transform(Transform::from_translation(
                                    Vec3::new(-50., 150., 1.),
                                )),
                            ));
                        });
                }
            });
    }
}

const FIGHT_BOARD_SIZE: Vec2 = Vec2::new(850., 700.);
const CARD_SIZE: Vec2 = Vec2::new(350., 600.);
const CARD_TEXT_TRANSFORM: Transform = Transform::from_translation(Vec3::new(-100., 260., 2.));

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
