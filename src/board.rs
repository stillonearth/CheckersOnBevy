use bevy::ecs::event::*;
use bevy::ecs::schedule::ShouldRun;
use bevy::pbr::*;
use bevy::{app::AppExit, prelude::*};
use bevy_mod_picking::*;

use crate::materials;
use crate::pieces;

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

#[derive(Component)]
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
    square: &Square,
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

fn run_if_square_is_selected(selected_square: Res<SelectedSquare>) -> ShouldRun {
    if selected_square.entity != None {
        return ShouldRun::Yes;
    };

    return ShouldRun::No;
}

fn run_if_piece_is_selected(selected_piece: Res<SelectedPiece>) -> ShouldRun {
    if selected_piece.entity != None {
        return ShouldRun::Yes;
    };

    return ShouldRun::No;
}

fn select_piece(
    mut commands: Commands,
    mut turn: ResMut<PlayerTurn>,
    mut selected_square: ResMut<SelectedSquare>,
    mut selected_piece: ResMut<SelectedPiece>,
    square_query: Query<(Entity, &Square)>,
    pieces_query: Query<(Entity, &pieces::Piece)>,
) {
    let square = square_query.get(selected_square.entity.unwrap()).unwrap().1;
    let pieces: Vec<pieces::Piece> = pieces_query.iter().map(|(_, piece)| *piece).collect();

    let (new_entity, new_piece) = find_piece_by_square(square, &pieces_query);
    let (_old_entity, old_piece) = find_piece_by_entity(selected_piece.entity, &pieces_query);

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

        commands.entity(_old_entity.unwrap()).insert(op);

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
    let chosen_square = filter_just_selected_event(picking_events);
    if chosen_square != None {
        selected_square.entity = chosen_square;
    }
}

fn highlight_square(
    selected_square: Res<SelectedSquare>,
    square_materials: Res<materials::Materials>,
    mut query: Query<(Entity, &Square, &mut Handle<StandardMaterial>)>,
) {
    for (entity, square, mut material) in query.iter_mut() {
        if Some(entity) == selected_square.entity {
            *material = square_materials.selected_color.clone();
        } else if square.color() == materials::Color::White {
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
            .init_resource::<PlayerTurn>()
            .add_startup_system(create_board.system())
            .add_system(event_square_selected.label("event_square_selected"))
            .add_system(
                select_piece
                    .label("select_piece")
                    .with_run_criteria(run_if_square_is_selected)
                    .after("event_square_selected"),
            )
            .add_system(
                despawn_taken_pieces
                    .label("despawn_taken_pieces")
                    .after("select_piece"),
            )
            .add_system(check_game_termination.system())
            .add_system(highlight_square.system())
            .add_plugin(pieces::PiecesPlugin);
    }
}

// ---
// TODOs
// ---

// Rust Generics
// Bevy ECS pattern
