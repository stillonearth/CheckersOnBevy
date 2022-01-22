use bevy::pbr::*;
use bevy::prelude::*;
use bevy::utils::Duration;
use bevy_tweening::*;

use crate::animations;
use crate::board;
use crate::materials;
use std::vec::*;

// ---
// Events
// ---
pub struct EventPieceMove(pub Entity);

// ---
// Components
// ---

#[derive(Component, Debug, Copy, Clone, PartialEq)]
pub struct Piece {
    pub color: materials::Color,
    pub y: u8,
    pub x: u8,
}

pub enum MoveType {
    Invalid,
    JumpOver,
    Regular,
}

impl Piece {
    pub fn move_to_square(&mut self, square: board::Square) {
        self.x = square.x;
        self.y = square.y;
    }

    fn translation(&self) -> Vec3 {
        let v1 = Vec3::new(self.x as f32, 0.1, self.y as f32);
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

    pub fn is_move_valid(&self, new_square: board::Square, pieces: &Vec<Piece>) -> MoveType {
        let is_square_occopied = pieces
            .iter()
            .filter(|p| p.x == new_square.x && p.y == new_square.y)
            .count()
            == 1;

        if is_square_occopied {
            return MoveType::Invalid;
        }

        let collision_count =
            self.is_path_empty((self.x, self.y), (new_square.x, new_square.y), pieces);

        // move to empty square
        if collision_count == 0 {
            let horizontal_move =
                (self.x as i8 - new_square.x as i8).abs() == 1 && (self.y == new_square.y);
            let vertical_move =
                (self.y as i8 - new_square.y as i8).abs() == 1 && (self.x == new_square.x);
            let diagonal_move = (self.y as i8 - new_square.y as i8).abs()
                == (self.x as i8 - new_square.x as i8).abs()
                && (self.x as i8 - new_square.x as i8).abs() == 1;

            if horizontal_move || vertical_move || diagonal_move {
                return MoveType::Regular;
            } else {
                return MoveType::Invalid;
            }
        } else if collision_count == 1 {
            let horizontal_move =
                (self.x as i8 - new_square.x as i8).abs() == 2 && (self.y == new_square.y);
            let vertical_move =
                (self.y as i8 - new_square.y as i8).abs() == 2 && (self.x == new_square.x);
            let diagonal_move = (self.y as i8 - new_square.y as i8).abs()
                == (self.x as i8 - new_square.x as i8).abs()
                && (self.x as i8 - new_square.x as i8).abs() == 2;
            if horizontal_move || vertical_move || diagonal_move {
                return MoveType::JumpOver;
            } else {
                return MoveType::Invalid;
            }
        } else {
            return MoveType::Invalid;
        }
    }

    pub fn is_path_empty(&self, begin: (u8, u8), end: (u8, u8), pieces: &Vec<Piece>) -> u8 {
        let mut collision_count: u8 = 0;
        // Same column
        if begin.0 == end.0 {
            for piece in pieces {
                if piece.x == begin.0
                    && ((piece.y > begin.1 && piece.y < end.1)
                        || (piece.y > end.1 && piece.y < begin.1))
                {
                    collision_count += 1;
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
                    collision_count += 1;
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
                    collision_count += 1;
                }
            }
        }

        return collision_count;
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

pub type Position = (u8, u8);
pub fn white_start_positions() -> Vec<Position> {
    let mut positions: Vec<Position> = Vec::new();

    for i in 0..3 {
        for j in 5..8 {
            let p: Position = (i as u8, j as u8);
            positions.push(p);
        }
    }

    return positions;
}

pub fn black_start_positions() -> Vec<Position> {
    let mut positions: Vec<Position> = Vec::new();

    for i in 5..8 {
        for j in 0..3 {
            let p: Position = (i as u8, j as u8);
            positions.push(p);
        }
    }

    return positions;
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
    for position in white_start_positions() {
        spawn_cp(
            &mut commands,
            square_materials.white_color.clone(),
            cp_handle.clone(),
            position,
            materials::Color::White,
        );
    }

    // spawn blacks
    for position in black_start_positions() {
        spawn_cp(
            &mut commands,
            square_materials.black_color.clone(),
            cp_handle.clone(),
            position,
            materials::Color::Black,
        );
    }
}

fn spawn_cp(
    commands: &mut Commands,
    material: Handle<StandardMaterial>,
    mesh: Handle<Mesh>,
    position: Position,
    piece_color: materials::Color,
) {
    let piece = Piece {
        color: piece_color,
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

        commands.entity(entity).insert(Animator::new(
            EaseFunction::QuadraticInOut,
            TweeningType::Once {
                duration: Duration::from_millis(800),
            },
            animations::TransformPositionWithYJumpLens {
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
            // .init_resource::<AsyncComputeTaskPool>()
            .add_plugin(TweeningPlugin)
            .add_system(highlight_piece.system())
            .add_system(event_piece_moved.system());
    }
}
