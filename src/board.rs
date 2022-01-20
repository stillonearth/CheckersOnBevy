use bevy::ecs::event::*;
use bevy::ecs::schedule::ShouldRun;
use bevy::pbr::*;
use bevy::{app::AppExit, prelude::*};
use bevy_mod_picking::*;

use crate::materials;
use crate::pieces;
use anyhow::Result;

// Entities

#[derive(Default)]
pub struct SelectedSquare {
    entity: Option<Entity>,
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

pub struct PlayerTurn(pub materials::Color);

impl Default for PlayerTurn {
    fn default() -> Self {
        Self(materials::Color::White)
    }
}

impl PlayerTurn {
    pub fn change(&mut self) {
        self.0 = match self.0 {
            materials::Color::White => materials::Color::Black,
            materials::Color::Black => materials::Color::White,
        }
    }
}

#[derive(Component)]
struct Taken;

// ---
// Components
// ---

#[derive(Component, Copy, Clone, Debug)]
pub struct Square {
    pub x: u8,
    pub y: u8,
}
impl Square {
    fn color(&self) -> materials::Color {
        if (self.x + self.y + 1) % 2 == 0 {
            materials::Color::White
        } else {
            materials::Color::Black
        }
    }
}

// ---
// Systems
// ---

pub fn create_board(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    square_materials: Res<materials::Materials>,
) {
    // Add meshes and materials
    let mesh = meshes.add(Mesh::from(shape::Plane { size: 1. }));

    // Spawn 64 squares
    for i in 0..8 {
        for j in 0..8 {
            let square = Square { x: i, y: j };
            let material = if square.color() == materials::Color::White {
                square_materials.white_color.clone()
            } else {
                square_materials.black_color.clone()
            };

            let bundle = PbrBundle {
                mesh: mesh.clone(),
                material: material,
                transform: Transform::from_translation(Vec3::new(i as f32, 0., j as f32)),
                ..Default::default()
            };

            commands
                .spawn_bundle(bundle)
                .insert_bundle(PickableBundle::default())
                .insert(square);
        }
    }
}

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
    square: Square,
    pieces_query: &Query<(Entity, &pieces::Piece)>,
) -> (Option<Entity>, Option<pieces::Piece>) {
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
    pieces_query: &Query<(Entity, &pieces::Piece)>,
) -> (Option<Entity>, Option<pieces::Piece>) {
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
    square_query: &Query<(Entity, &Square)>,
) -> (Option<Entity>, Option<Square>) {
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

fn check_game_termination(
    turn: Res<PlayerTurn>,
    mut app_exit_events: ResMut<Events<AppExit>>,
    pieces_query: Query<(Entity, &mut pieces::Piece)>,
) {
    let pieces: Vec<pieces::Piece> = pieces_query.iter().map(|(_, piece)| *piece).collect();

    // Game end condition check
    let number_of_whites = pieces
        .iter()
        .filter(|p| p.color == materials::Color::White)
        .count();
    let number_of_blacks = pieces
        .iter()
        .filter(|p| p.color == materials::Color::Black)
        .count();

    if number_of_whites == 0 || number_of_blacks == 0 {
        println!(
            "{} won! Thanks for playing!",
            match turn.0 {
                materials::Color::White => "Black",
                materials::Color::Black => "White",
            }
        );
        app_exit_events.send(AppExit);
    }
}

fn select_piece(
    mut commands: Commands,
    mut turn: ResMut<PlayerTurn>,
    mut selected_square: ResMut<SelectedSquare>,
    mut selected_piece: ResMut<SelectedPiece>,
    mut event_piece_move: EventWriter<pieces::EventPieceMove>,
    square_query: Query<(Entity, &Square)>,
    pieces_query: Query<(Entity, &pieces::Piece)>,
) {
    let (_, square) = find_square_by_entity(selected_square.entity, &square_query);

    let pieces: Vec<pieces::Piece> = pieces_query.iter().map(|(_, piece)| *piece).collect();

    if square.is_none() {
        return;
    }
    let square = square.unwrap();

    let (new_entity, new_piece) = find_piece_by_square(square, &pieces_query);
    let (old_entity, old_piece) = find_piece_by_entity(selected_piece.entity, &pieces_query);

    // Nothing has been selected before
    if old_piece == None && new_entity != None && new_piece.unwrap().color == turn.0 {
        selected_piece.entity = new_entity;
        return;
    }

    if old_piece == None {
        return;
    }

    let mut op = old_piece.unwrap();

    // Another piece currently selected
    if op.is_move_valid(square, &pieces) {
        op.move_to_square(square);

        // this updates component
        let e = selected_piece.entity.unwrap();
        commands.entity(e).insert(op);
        event_piece_move.send(pieces::EventPieceMove(e));

        selected_piece.deselect();
        selected_square.deselect();

        if !new_entity.is_none() {
            commands.entity(new_entity.unwrap()).insert(Taken);
        }

        turn.change();
    } else {
        selected_piece.entity = new_entity;
    }
}

fn despawn_taken_pieces(mut commands: Commands, query: Query<(Entity, &Taken)>) {
    for (entity, _taken) in query.iter() {
        commands.entity(entity).despawn();
    }
}

fn event_square_selected(
    mut selected_square: ResMut<SelectedSquare>,
    picking_events: EventReader<PickingEvent>,
) {
    selected_square.entity = filter_just_selected_event(picking_events);
}

// ---
// Plugins
// ---

pub struct BoardPlugin;
impl Plugin for BoardPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SelectedSquare>()
            .init_resource::<SelectedPiece>()
            .init_resource::<PlayerTurn>()
            // Initializtion
            .add_startup_system(create_board)
            // Game Logic logic
            .add_system(event_square_selected)
            .add_system(select_piece)
            .add_system(despawn_taken_pieces)
            .add_system(check_game_termination)
            // Events
            .add_event::<pieces::EventPieceMove>()
            .add_plugin(pieces::PiecesPlugin);
    }
}

// ---
// TODOs
// ---

// Rust Generics
// Bevy ECS pattern
