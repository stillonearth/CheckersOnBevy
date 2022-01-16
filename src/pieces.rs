use bevy::pbr::*;
use bevy::prelude::*;

use crate::board;
use crate::materials;

// ---
// Entities
// ---

#[derive(Default)]
pub struct SelectedPiece {
    pub entity: Option<Entity>,
}

// ---
// Components
// ---

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum PieceType {
    Man,
    // King,
}

#[derive(Component, Debug)]
pub struct Piece {
    pub color: materials::Color,
    pub piece_type: PieceType,
    pub y: u8,
    pub x: u8,
}

impl Piece {
    pub fn move_to_square(&mut self, square: &board::Square) {
        self.x = square.x;
        self.y = square.y;
    }

    fn translation(&self) -> Vec3 {
        let v1 = Vec3::new(self.x as f32, 0.2, self.y as f32);
        return v1;
    }

    pub fn transform(&self) -> Transform {
        // Translation
        let mut transform = Transform::from_translation(self.translation());

        // Rotation
        transform.rotate(Quat::from_rotation_x(-1.57));
        if self.color == materials::Color::Black {
            transform.rotate(Quat::from_rotation_y(-1.57));
        } else {
            transform.rotate(Quat::from_rotation_y(1.57));
        }
        transform.rotate(Quat::from_rotation_z(3.14));

        // Scale
        transform.apply_non_uniform_scale(Vec3::new(0.02, 0.02, 0.02));

        return transform;
    }
}

// ---
// Systems
// ---

pub fn create_pieces(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    square_materials: Res<materials::Materials>,
) {
    let cp_handle = asset_server.load("microsoft.glb#Mesh0/Primitive0");

    // spawn whites
    for i in 0..3 {
        for j in 5..8 {
            spawn_cp(
                &mut commands,
                square_materials.white_color.clone().clone(),
                cp_handle.clone(),
                (i, j),
                materials::Color::White,
            );
        }
    }

    // spawn blacks
    for i in 5..8 {
        for j in 0..3 {
            spawn_cp(
                &mut commands,
                square_materials.black_color.clone().clone(),
                cp_handle.clone(),
                (i, j),
                materials::Color::Black,
            );
        }
    }
}

// fn cp_pi

fn spawn_cp(
    commands: &mut Commands,
    material: Handle<StandardMaterial>,
    mesh: Handle<Mesh>,
    position: (u8, u8),
    piece_color: materials::Color,
) {
    let piece = Piece {
        color: piece_color,
        piece_type: PieceType::Man,
        x: position.0,
        y: position.1,
    };

    let bundle = PbrBundle {
        mesh: mesh,
        material: material.clone(),
        transform: piece.transform(),
        ..Default::default()
    };

    commands.spawn_bundle(bundle).insert(piece);
}

fn move_pieces(time: Res<Time>, mut query: Query<(Entity, &Piece, &mut Transform)>) {
    for (_, piece, mut transform) in query.iter_mut() {
        let direction = piece.translation() - transform.translation;
        let delta = direction.normalize() * time.delta_seconds();

        if direction.length() > 0.05 {
            transform.translation += delta;
        }
    }
}

fn highlight_piece(
    selected_piece: Res<SelectedPiece>,
    square_materials: Res<materials::Materials>,
    mut query: Query<(Entity, &Piece, &mut Handle<StandardMaterial>)>,
) {
    for (entity, piece, mut material) in query.iter_mut() {
        if Some(entity) == selected_piece.entity {
            *material = square_materials.selected_color.clone();
        } else if piece.color == materials::Color::White {
            *material = square_materials.white_color.clone();
        } else {
            *material = square_materials.black_color.clone();
        }
    }
}

// ---
// Plugins
// ---

pub struct PiecesPlugin;
impl Plugin for PiecesPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(create_pieces.system())
            .add_system(highlight_piece.system())
            .add_system(move_pieces.system());
    }
}
