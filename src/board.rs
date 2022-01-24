use bevy::ecs::event::*;
use bevy::pbr::*;
use bevy::{app::AppExit, prelude::*};
use bevy_mod_picking::*;

use crate::game;
use crate::materials;
use crate::pieces;

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
    pieces_query: &Query<(Entity, &mut game::Piece)>,
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
    mut pieces_query: Query<(Entity, &mut game::Piece)>,
) -> (Option<Entity>, Option<game::Piece>) {
    if entity == None {
        return (None, None);
    }

    match pieces_query
        .iter_mut()
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

// ---
// Systems
// ---

pub fn create_board(
    mut commands: Commands,
    game: Res<&'static mut game::Game>,
    mut meshes: ResMut<Assets<Mesh>>,
    square_materials: Res<materials::Materials>,
) {
    // Add meshes and materials
    let mesh = meshes.add(Mesh::from(shape::Plane { size: 1. }));

    for square in game.squares.iter() {
        let material = if square.color() == game::Color::White {
            square_materials.white_color.clone()
        } else {
            square_materials.black_color.clone()
        };

        let bundle = PbrBundle {
            mesh: mesh.clone(),
            material: material,
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
    mut commands: Commands,
    // game logic
    mut game: ResMut<&'static mut game::Game>,
    // bevy game entities
    mut selected_square: ResMut<SelectedSquare>,
    mut selected_piece: ResMut<SelectedPiece>,
    // events
    mut event_piece_move: EventWriter<pieces::EventPieceMove>,
    // queries
    square_query: Query<(Entity, &game::Square)>,
    pieces_query: Query<(Entity, &mut game::Piece)>,
) {
    let (_, new_square) = find_square_by_entity(selected_square.entity, &square_query);

    if new_square.is_none() {
        return;
    }

    let (new_entity, new_piece) = find_piece_by_square(new_square.unwrap(), &pieces_query);
    let (old_entity, old_piece) = find_piece_by_entity(selected_piece.entity, pieces_query);

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

    let piece: &mut game::Piece = &mut old_piece.unwrap();
    let entity = old_entity.unwrap();

    let (move_type, state, termination) = game.step(piece, new_square.unwrap());
    // Check whether game move was valid

    info!(
        "move_type: {:?}\tstate: {:?}\ttermination: {:?}",
        move_type, state, termination
    );

    match move_type {
        game::MoveType::Invalid => {
            if new_piece != None && new_piece.unwrap().color == game.state.turn.color {
                selected_piece.entity = new_entity;
            }
        }
        game::MoveType::JumpOver => {
            commands.entity(entity).insert(*piece);
            event_piece_move.send(pieces::EventPieceMove(entity));
        }
        game::MoveType::Regular => {
            selected_piece.deselect();
            selected_square.deselect();
            commands.entity(entity).insert(*piece);
            event_piece_move.send(pieces::EventPieceMove(entity));
        }
    }
}

fn event_square_selected(
    mut selected_square: ResMut<SelectedSquare>,
    picking_events: EventReader<PickingEvent>,
) {
    selected_square.entity = filter_just_selected_event(picking_events);
}

fn check_game_termination(
    game: Res<&'static mut game::Game>,
    mut event_app_exit: ResMut<Events<AppExit>>,
) {
    if !game.is_changed() {
        return;
    }
    // Check whether game has ended
    match game.check_termination() {
        game::GameTermination::Black => {
            println!("Black won! Thanks for playing!");
            event_app_exit.send(AppExit);
        }
        game::GameTermination::White => {
            println!("White won! Thanks for playing!");
            event_app_exit.send(AppExit);
        }
        _ => {}
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
            .add_system(event_square_selected)
            .add_system(click_square)
            .add_system(check_game_termination)
            .add_event::<pieces::EventPieceMove>()
            .add_plugin(pieces::PiecesPlugin);
    }
}
