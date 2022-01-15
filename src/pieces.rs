use bevy::pbr::*;
use bevy::prelude::*;
// ---
// Entities
// ---

// ---
// Components
// ---

#[derive(Clone, Copy, PartialEq)]
pub enum PieceColor {
    White,
    Black,
}

#[derive(Clone, Copy, PartialEq)]
pub enum PieceType {
    Man,
    King,
}

#[derive(Component)]
pub struct Piece {
    pub color: PieceColor,
    pub piece_type: PieceType,
    pub y: u8,
    pub x: u8,
}

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
        for j in 5..8 {
            spawn_cp(
                &mut commands,
                white_material.clone(),
                cp_handle.clone(),
                (i, j),
                PieceColor::White,
            );
        }
    }

    // spawn blacks
    for i in 5..8 {
        for j in 0..3 {
            spawn_cp(
                &mut commands,
                black_material.clone(),
                cp_handle.clone(),
                (i, j),
                PieceColor::Black,
            );
        }
    }
}

fn spawn_cp(
    commands: &mut Commands,
    material: Handle<StandardMaterial>,
    mesh: Handle<Mesh>,
    position: (u8, u8),
    piece_color: PieceColor,
) {
    let position_vec3 = Vec3::new(
        (position.0 as f32) + 0.5,
        0.,
        ((position.1 as f32) as f32) - 3.5,
    );

    commands
        .spawn_bundle((
            Transform::from_translation(position_vec3),
            GlobalTransform::identity(),
        ))
        .with_children(|parent| {
            parent
                .spawn_bundle(PbrBundle {
                    mesh: mesh,
                    material: material.clone(),
                    transform: {
                        let mut transform = Transform::from_translation(Vec3::new(-0.5, 0.05, 3.5));
                        transform.rotate(Quat::from_rotation_x(-1.57));
                        // TODO: mave expressive
                        if piece_color == PieceColor::White {
                            transform.rotate(Quat::from_rotation_y(1.57));
                        } else {
                            transform.rotate(Quat::from_rotation_y(-1.57));
                        }
                        transform.rotate(Quat::from_rotation_z(3.14));

                        transform.apply_non_uniform_scale(Vec3::new(0.02, 0.02, 0.02));
                        transform
                    },
                    ..Default::default()
                })
                .insert(Piece {
                    color: piece_color,
                    piece_type: PieceType::Man,
                    x: position.0,
                    y: position.1,
                });
        });
}

fn move_pieces(time: Res<Time>, mut query: Query<(&mut Transform, &Piece)>) {
    for (mut transform, piece) in query.iter_mut() {
        let direction = Vec3::new(piece.x as f32, 0.0, piece.y as f32) - transform.translation;

        if direction.length() > 0.1 {
            transform.translation += direction.normalize() * time.delta_seconds();
        }
    }
}

// ---
// Plugins
// ---

pub struct PiecesPlugin;
impl Plugin for PiecesPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(create_pieces.system());
        // .add_system(move_pieces.system());
    }
}
