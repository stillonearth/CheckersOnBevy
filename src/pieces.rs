use bevy::pbr::*;
use bevy::prelude::*;
// ---
// Entities
// ---

// ---
// Components
// ---

// ---
// Systems
// ---

pub fn create_pieces(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let cp_handle = asset_server.load("microsoft.glb#Mesh0/Primitive0");

    let white_material = materials.add(Color::rgb(0.9, 0.8, 0.8).into());
    let black_material = materials.add(Color::rgb(0., 0.2, 0.2).into());

    // spawn whites
    for i in 0..3 {
        for j in 0..3 {
            spawn_cp(
                &mut commands,
                white_material.clone(),
                cp_handle.clone(),
                Vec3::new(4. - (i as f32), 0., -(j as f32)),
            );
        }
    }

    // spawn blacks
    for i in 0..3 {
        for j in 0..3 {
            spawn_cp(
                &mut commands,
                black_material.clone(),
                cp_handle.clone(),
                Vec3::new(-3. + (i as f32), 0., -7. + (j as f32)),
            );
        }
    }
}

pub fn spawn_cp(
    commands: &mut Commands,
    material: Handle<StandardMaterial>,
    mesh: Handle<Mesh>,
    position: Vec3,
) {
    commands
        .spawn_bundle((
            Transform::from_translation(position),
            GlobalTransform::identity(),
        ))
        .with_children(|parent| {
            parent.spawn_bundle(PbrBundle {
                mesh: mesh,
                material: material.clone(),
                transform: {
                    let mut transform = Transform::from_translation(Vec3::new(-0.5, 0.05, 3.5));
                    transform.rotate(Quat::from_rotation_x(-1.57));
                    transform.rotate(Quat::from_rotation_z(3.14));

                    transform.apply_non_uniform_scale(Vec3::new(0.02, 0.02, 0.02));
                    transform
                },
                ..Default::default()
            });
        });
}

// ---
// Plugins
// ---
