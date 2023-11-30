use bevy::{prelude::*, sprite::MaterialMesh2dBundle};

use crate::core_logic::{AbilitySlotType, AbilitySlots};

#[derive(Debug, Component)]
pub struct AbilitySlotsSection {
    model: Entity,
}

impl AbilitySlotsSection {
    pub fn new(model: Entity) -> Self {
        Self { model }
    }
}

pub fn sync_to_models(
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

        if let Ok(model_slots) = slots.get(section.model) {
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
        };
    }
}
