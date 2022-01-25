use bevy::pbr::*;
use bevy::prelude::*;
use bevy::utils::Duration;
use bevy_tweening::*;

use std::sync::Arc;
use std::sync::Mutex;

use crate::animations;
use crate::board;
use crate::game;
use crate::materials;

// ---
// Events
// ---
pub struct EventPieceMove(pub Entity);

// ---
// Systems
// ---

pub fn create_pieces(
    mut commands: Commands,
    game: Res<Arc<Mutex<game::Game>>>,
    asset_server: Res<AssetServer>,
    square_materials: Res<materials::Materials>,
) {
    let cp_handle = asset_server.load("microsoft.glb#Mesh0/Primitive0");

    let game = game.lock().unwrap();

    for piece in game.state.pieces.iter() {
        let bundle = PbrBundle {
            mesh: cp_handle.clone(),
            material: match piece.color {
                game::Color::Black => square_materials.black_color.clone(),
                game::Color::White => square_materials.white_color.clone(),
            },
            transform: model_transform(*piece),
            ..Default::default()
        };
        commands.spawn_bundle(bundle).insert(*piece);
    }
}

pub fn model_transform(piece: game::Piece) -> Transform {
    // Translation
    let mut transform = Transform::from_translation(piece.translation());

    // Rotation
    transform.rotate(Quat::from_rotation_x(-1.57));
    if piece.color == game::Color::Black {
        transform.rotate(Quat::from_rotation_y(-1.57));
    } else {
        transform.rotate(Quat::from_rotation_y(1.57));
    }
    transform.rotate(Quat::from_rotation_z(3.14));

    // Scale
    transform.apply_non_uniform_scale(Vec3::new(0.02, 0.02, 0.02));

    return transform;
}

fn event_piece_moved(
    mut commands: Commands,
    mut picking_events: EventReader<EventPieceMove>,
    mut query: Query<(Entity, &game::Piece, &Transform)>,
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
    // game: Res<Arc<Mutex<game::Game>>>,
    // turn: Res<game::PlayerTurn>,
    selected_piece: Res<board::SelectedPiece>,
    square_materials: Res<materials::Materials>,
    mut query: Query<(Entity, &game::Piece, &mut Handle<StandardMaterial>)>,
) {
    // info!("game {:?}", game.state.turn.color);
    // info!("turn {:?}", turn.color);

    for (entity, piece, mut material) in query.iter_mut() {
        if Some(entity) == selected_piece.entity {
            *material = square_materials.selected_color.clone();
        } else if piece.color == game::Color::White {
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
