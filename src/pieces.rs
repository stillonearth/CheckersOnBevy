use bevy::pbr::*;
use bevy::prelude::*;
use bevy::utils::Duration;
use bevy_tweening::*;

use crate::board;
use crate::materials;

// ---
// Events
// ---
pub struct EventPieceMove(pub Entity);

// ---
// Components
// ---

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum PieceType {
    Man,
    // King,
}

#[derive(Component, Debug, Copy, Clone, PartialEq)]
pub struct Piece {
    pub color: materials::Color,
    pub piece_type: PieceType,
    pub y: u8,
    pub x: u8,
}

impl Piece {
    pub fn move_to_square(&mut self, square: board::Square) {
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

    pub fn is_move_valid(&self, new_square: board::Square, pieces: &Vec<Piece>) -> bool {
        // If there's a piece of the same color in the same square, it can't move
        if color_of_square((new_square.x, new_square.y), &pieces) == Some(self.color) {
            return false;
        }

        match self.piece_type {
            PieceType::Man => {
                let horizontal_move =
                    (self.x as i8 - new_square.x as i8).abs() > 0 && (self.y == new_square.y);
                let vertical_move =
                    (self.y as i8 - new_square.y as i8).abs() > 0 && (self.x == new_square.x);
                let diagonal_move = (self.y as i8 - new_square.y as i8).abs()
                    == (self.x as i8 - new_square.x as i8).abs();
                let path_nonblocking =
                    self.is_path_empty((self.x, self.y), (new_square.x, new_square.y), pieces);

                return (horizontal_move || vertical_move || diagonal_move) && path_nonblocking;
            }
        }
    }

    pub fn is_path_empty(&self, begin: (u8, u8), end: (u8, u8), pieces: &Vec<Piece>) -> bool {
        // Same column
        if begin.0 == end.0 {
            for piece in pieces {
                if piece.x == begin.0
                    && ((piece.y > begin.1 && piece.y < end.1)
                        || (piece.y > end.1 && piece.y < begin.1))
                {
                    return false;
                }
            }
        }
        // Same row
        if begin.1 == end.1 {
            for piece in pieces {
                if piece.y == begin.1
                    && ((piece.x > begin.0 && piece.x < end.0)
                        || (piece.x > end.0 && piece.x < begin.0))
                {
                    return false;
                }
            }
        }

        // Diagonals
        let x_diff = (begin.0 as i8 - end.0 as i8).abs();
        let y_diff = (begin.1 as i8 - end.1 as i8).abs();
        if x_diff == y_diff {
            for i in 1..x_diff {
                let pos = if begin.0 < end.0 && begin.1 < end.1 {
                    // left bottom - right top
                    (begin.0 + i as u8, begin.1 + i as u8)
                } else if begin.0 < end.0 && begin.1 > end.1 {
                    // left top - right bottom
                    (begin.0 + i as u8, begin.1 - i as u8)
                } else if begin.0 > end.0 && begin.1 < end.1 {
                    // right bottom - left top
                    (begin.0 - i as u8, begin.1 + i as u8)
                } else {
                    // begin.0 > end.0 && begin.1 > end.1
                    // right top - left bottom
                    (begin.0 - i as u8, begin.1 - i as u8)
                };

                if color_of_square(pos, pieces).is_some() {
                    return false;
                }
            }
        }

        true
    }
}

// ---
// Game Logic
// ---

/// Returns None if square is empty, returns a Some with the color if not
pub fn color_of_square(pos: (u8, u8), pieces: &Vec<Piece>) -> Option<materials::Color> {
    for piece in pieces {
        if piece.x == pos.0 && piece.y == pos.1 {
            return Some(piece.color);
        }
    }
    None
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

fn event_piece_moved(
    mut commands: Commands,
    mut picking_events: EventReader<EventPieceMove>,
    mut query: Query<(Entity, &Piece, &Transform)>,
) {
    for event in picking_events.iter() {
        let (entity, piece, transform) = query.get_mut(event.0).unwrap();
        // result.unwrap()
        commands.entity(entity).insert(Animator::new(
            // Use a quadratic easing on both endpoints
            EaseFunction::QuadraticInOut,
            // Loop animation back and forth over 1 second, with a 0.5 second
            // pause after each cycle (start -> end -> start).
            TweeningType::Once {
                duration: Duration::from_secs(1),
            },
            TransformPositionLens {
                start: transform.translation,
                end: piece.translation(),
            },
        ));
    }
}

fn highlight_piece(
    selected_piece: Res<board::SelectedPiece>,
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
            .add_plugin(TweeningPlugin)
            .add_system(highlight_piece.system())
            .add_system(event_piece_moved.system());
    }
}
