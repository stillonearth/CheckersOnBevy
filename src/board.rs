use bevy::ecs::event::*;
use bevy::ecs::query::QueryIter;
use bevy::pbr::*;
use bevy::{app::AppExit, prelude::*};
use bevy_mod_picking::*;

use bevy::utils::Duration;
use bevy_tweening::*;

use crate::animations;
use crate::board;
use crate::game;
use crate::materials;

// ---
// Resources
// ---

#[derive(Default)]
pub struct SelectedSquare {
    pub entity: Option<Entity>,
}

impl SelectedSquare {
    pub fn deselect(&mut self) {
        self.entity = None;
    }
}

#[derive(Default)]
pub struct SelectedPiece {
    pub entity: Option<Entity>,
}

impl SelectedPiece {
    pub fn deselect(&mut self) {
        self.entity = None;
    }
}

// ---
// Events
// ---
pub struct EventPieceMove(pub Entity);

// ---
// Helpers
// ---
fn filter_just_selected_event(mut event_reader: EventReader<PickingEvent>) -> Option<Entity> {
    for event in event_reader.iter() {
        match event {
            PickingEvent::Selection(selection_event) => match selection_event {
                SelectionEvent::JustSelected(selection_event) => {
                    return Some(*selection_event);
                }
                _ => {}
            },
            _ => {}
        }
    }

    return None;
}

fn find_piece_by_square(
    square: game::Square,
    pieces_query: &Query<(Entity, &game::Piece)>,
) -> (Option<Entity>, Option<game::Piece>) {
    match pieces_query
        .iter()
        .filter(|(_, p)| p.x == square.x && p.y == square.y)
        .nth(0)
    {
        // Square  hold piece
        Some((e, p)) => {
            return (Some(e), Some(*p));
        }

        // Square doesn't hold piece
        _ => return (None, None),
    };
}

fn find_piece_by_entity(
    entity: Option<Entity>,
    pieces_query: &Query<(Entity, &game::Piece)>,
) -> (Option<Entity>, Option<game::Piece>) {
    if entity == None {
        return (None, None);
    }

    match pieces_query
        .iter()
        .filter(|(e, _)| e == &entity.unwrap())
        .nth(0)
    {
        // Square  hold piece
        Some((e, p)) => {
            return (Some(e), Some(*p));
        }

        // Square doesn't hold piece
        _ => return (None, None),
    };
}

fn find_square_by_entity(
    entity: Option<Entity>,
    square_query: &Query<(Entity, &game::Square)>,
) -> (Option<Entity>, Option<game::Square>) {
    if entity == None {
        return (None, None);
    }

    match square_query
        .iter()
        .filter(|(e, _)| e == &entity.unwrap())
        .nth(0)
    {
        // Square  hold piece
        Some((e, p)) => {
            return (Some(e), Some(*p));
        }

        // Square doesn't hold piece
        _ => return (None, None),
    };
}

pub fn piece_translation(piece: game::Piece) -> Vec3 {
    let v1 = Vec3::new(piece.x as f32, 0.1, piece.y as f32);
    return v1;
}

// ---
// Systems
// ---

pub fn create_board(
    mut commands: Commands,
    game: Res<game::Game>,
    mut meshes: ResMut<Assets<Mesh>>,
    square_materials: Res<materials::Materials>,
) {
    // Add meshes and materials
    let mesh = meshes.add(Mesh::from(shape::Plane { size: 1. }));
    let game = game;

    for square in game.squares.iter() {
        let material = if square.color() == game::Color::White {
            square_materials.white_color.clone()
        } else {
            square_materials.black_color.clone()
        };

        let bundle = PbrBundle {
            mesh: mesh.clone(),
            material,
            transform: Transform::from_translation(Vec3::new(square.x as f32, 0., square.y as f32)),
            ..Default::default()
        };

        commands
            .spawn_bundle(bundle)
            .insert_bundle(PickableBundle::default())
            .insert(*square);
    }
}

fn click_square(
    mut game: ResMut<game::Game>,
    // bevy game entities
    mut selected_square: ResMut<SelectedSquare>,
    mut selected_piece: ResMut<SelectedPiece>,
    // queries
    square_query: Query<(Entity, &game::Square)>,
    pieces_query: Query<(Entity, &game::Piece)>,
) {
    let (_, new_square) = find_square_by_entity(selected_square.entity, &square_query);

    if new_square.is_none() {
        return;
    }

    let (new_entity, new_piece) = find_piece_by_square(new_square.unwrap(), &pieces_query);
    let (_old_entity, old_piece) = find_piece_by_entity(selected_piece.entity, &pieces_query);

    // Nothing has been selected before
    if selected_piece.entity == None
        && new_entity != None
        && new_piece.unwrap().color == game.state.turn.color
    {
        selected_piece.entity = new_entity;
        return;
    }

    if old_piece == None {
        return;
    }

    let piece = &mut old_piece.unwrap();
    let turn_color = game.state.turn.color;

    let (move_type, _new_state, _termination) = game.step(piece, new_square.unwrap());

    // Check whether game move was valid
    match move_type {
        game::MoveType::Invalid => {
            // chain move allowed for same piece
            if new_piece != None
                && new_piece.unwrap().color == turn_color
                && game.state.turn.chain_count == 0
            {
                selected_piece.entity = new_entity;
            }
        }
        game::MoveType::Regular => {
            selected_piece.deselect();
            selected_square.deselect();
        }
        _ => {}
    }
}

fn event_square_selected(
    mut selected_square: ResMut<SelectedSquare>,
    picking_events: EventReader<PickingEvent>,
) {
    selected_square.entity = filter_just_selected_event(picking_events);
}

fn check_game_termination(game: Res<game::Game>, mut event_app_exit: ResMut<Events<AppExit>>) {
    match game.check_termination() {
        game::GameTermination::Black | game::GameTermination::BlackMoveLimit => {
            // println!("Black won! Thanks for playing!");
            // event_app_exit.send(AppExit);
        }
        game::GameTermination::White | game::GameTermination::WhiteMoveLimit => {
            // println!("White won! Thanks for playing!");
            // event_app_exit.send(AppExit);
        }
        _ => {}
    }
}

// this should be done otherwise
fn update_entity_pieces(
    mut commands: Commands,
    game: Res<game::Game>,
    // events
    mut event_piece_move: EventWriter<EventPieceMove>,
    // queries
    query: Query<(Entity, &game::Piece)>,
) {
    for (e, p) in query.iter() {
        if !game.is_changed() {
            return;
        }

        let new_piece = game
            .state
            .pieces
            .iter()
            .filter(|_p| _p.id == p.id)
            .nth(0)
            .unwrap();

        if p.x != new_piece.x || p.y != new_piece.y {
            commands.entity(e).insert(*new_piece);
            event_piece_move.send(EventPieceMove(e));
        }
    }
}

pub fn create_pieces(
    mut commands: Commands,
    game: Res<game::Game>,
    asset_server: Res<AssetServer>,
    square_materials: Res<materials::Materials>,
) {
    let cp_handle = asset_server.load("microsoft.glb#Mesh0/Primitive0");

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
    let mut transform = Transform::from_translation(piece_translation(piece));

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
                duration: Duration::from_millis(500),
            },
            animations::TransformPositionWithYJumpLens {
                start: transform.translation,
                end: piece_translation(*piece),
            },
        ));
    }
}

fn highlight_piece(
    // game: Res<game::Game>,
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

pub struct BoardPlugin;
impl Plugin for BoardPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SelectedSquare>()
            .init_resource::<SelectedPiece>()
            .add_startup_system(create_board)
            .add_system(click_square.label("input"))
            .add_system(update_entity_pieces.after("input"))
            .add_system(event_square_selected)
            .add_system(check_game_termination)
            .add_event::<EventPieceMove>()
            .add_plugin(PiecesPlugin);
    }
}

pub struct PiecesPlugin;
impl Plugin for PiecesPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(create_pieces.system())
            .add_plugin(TweeningPlugin)
            .add_system(highlight_piece.system())
            .add_system(event_piece_moved.system());
    }
}
