use bevy::{prelude::*, sprite::MaterialMesh2dBundle};

use crate::core_logic::HasAbilities;

#[derive(Debug, Component)]
pub struct AbilitiesSection {
    pub model: Entity,
}

pub fn sync_to_models(
    mut commands: Commands,
    sections: Query<(Entity, Ref<AbilitiesSection>)>,
    abilities: Query<Ref<HasAbilities>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for (en, section) in sections.iter() {
        let should_spawn_text = section.is_added();

        if should_spawn_text {
            commands.entity(en).with_children(|parent| {
                parent.spawn(Text2dBundle {
                    text: Text::from_section(
                        "Abilities",
                        TextStyle {
                            font_size: 27.0,
                            color: Color::BLACK,
                            ..default()
                        },
                    )
                    .with_alignment(TextAlignment::Left),
                    transform: Transform::from_translation(Vec3::new(-40., 0., 1.)),
                    ..default()
                });
            });
        }
    }
}
