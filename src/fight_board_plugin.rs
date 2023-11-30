use bevy::{prelude::*, sprite::MaterialMesh2dBundle};

use crate::core_logic::{AbilitySlotType, AbilitySlots, Enemy, Fight, PlayerCharacter};

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

#[derive(Debug, Component)]
struct AbilitySlotsSection {
    model: Entity,
}

impl AbilitySlotsSection {
    fn new(model: Entity) -> Self {
        Self { model }
    }
}

const FIGHT_BOARD_SIZE: Vec2 = Vec2::new(850., 700.);
const CARD_SIZE: Vec2 = Vec2::new(350., 600.);
const CARD_TEXT_TRANSFORM: Transform = Transform::from_translation(Vec3::new(-100., 260., 2.));

// detect when `Fight` is added to the world
fn fight_added(
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

fn ability_slots_section_sync(
    mut commands: Commands,
    sections: Query<(Entity, Ref<AbilitySlotsSection>)>,
    slots: Query<Ref<AbilitySlots>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for (en, section) in sections.iter() {
        let should_spawn_text = section.is_added();

        if should_spawn_text {
            commands.entity(en).with_children(|parent| {
                parent.spawn(Text2dBundle {
                    text: Text::from_section(
                        "Ability Slots",
                        TextStyle {
                            font_size: 27.0,
                            color: Color::BLACK,
                            ..default()
                        },
                    )
                    .with_alignment(TextAlignment::Left),
                    transform: Transform::from_translation(Vec3::new(-10., 0., 1.)),
                    ..default()
                });
            });
        }

        let Ok(model_slots) = slots.get(section.model) else {
            continue;
        };

        let model_changed = model_slots.is_changed();
        let section_changed = section.is_changed() && (!section.is_added());
        let should_despawn = section_changed || model_changed;

        if should_despawn {
            commands.entity(en).clear_children();
        }

        let should_spawn = should_despawn || section.is_added();

        if should_spawn {
            commands.entity(en).with_children(|parent| {
                parent.spawn(Text2dBundle {
                    text: Text::from_section(
                        "Ability Slots",
                        TextStyle {
                            font_size: 27.0,
                            color: Color::BLACK,
                            ..default()
                        },
                    )
                    .with_alignment(TextAlignment::Left),
                    transform: Transform::from_translation(Vec3::new(-10., 0., 1.)),
                    ..default()
                });

                for (idx, slot) in model_slots.0.iter().enumerate() {
                    let color = match slot.tpe {
                        AbilitySlotType::WeaponAttack => Color::LIME_GREEN,
                        AbilitySlotType::ShieldDefend => Color::PINK,
                    };
                    parent.spawn(MaterialMesh2dBundle {
                        mesh: meshes
                            .add(shape::Quad::new(Vec2::new(35., 35.)).into())
                            .into(),
                        material: materials.add(ColorMaterial::from(color)),
                        transform: Transform::from_translation(Vec3::new(
                            -85. + 50. * (idx as f32),
                            -50.,
                            1.,
                        )),
                        ..default()
                    });
                }
            });
        }
    }
}

pub struct FightBoardPlugin;

impl Plugin for FightBoardPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (fight_added, ability_slots_section_sync).chain());
    }
}
