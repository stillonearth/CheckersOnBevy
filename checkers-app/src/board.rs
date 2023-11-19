use std::time::Duration;

use bevy::{app::AppExit, prelude::*};

use bevy_mod_picking::prelude::*;
use bevy_tweening::*;
use checkers_core::game;

use crate::*;

// ---
// Resources
// ---

#[derive(Resource)]
pub struct Materials {
    pub selected_color: Handle<StandardMaterial>,
    pub black_color: Handle<StandardMaterial>,
    pub white_color: Handle<StandardMaterial>,
    pub blue_color: Handle<StandardMaterial>,
}

impl FromWorld for Materials {
    fn from_world(world: &mut World) -> Self {
        let world = world.cell();
        let mut materials_asset = world
            .get_resource_mut::<Assets<StandardMaterial>>()
            .unwrap();

        Materials {
            selected_color: materials_asset.add(bevy::prelude::Color::rgb(0.9, 0.1, 0.1).into()),
            black_color: materials_asset.add(bevy::prelude::Color::rgb(0., 0.1, 0.1).into()),
            white_color: materials_asset.add(bevy::prelude::Color::rgb(1., 0.9, 0.9).into()),
            blue_color: materials_asset.add(bevy::prelude::Color::rgb(0.2, 0.2, 1.0).into()),
        }
    }
}

#[derive(Resource, Default)]
pub struct SelectedSquare {
    pub entity: Option<Entity>,
}

impl SelectedSquare {
    pub fn deselect(&mut self) {
        self.entity = None;
    }
}

#[derive(Resource, Default)]
pub struct SelectedPiece {
    pub entity: Option<Entity>,
}

impl SelectedPiece {
    pub fn deselect(&mut self) {
        self.entity = None;
    }
}

#[derive(PartialEq, Clone, Copy)]
pub enum Player {
    One,
    Two,
}

#[derive(Resource, Deref, DerefMut, PartialEq)]
pub struct InitialPlayer(pub Player);

// ---
// Events
// ---

#[derive(Event)]
pub struct EventPieceMove(pub Entity);

#[derive(Event)]
pub struct EventPieceOffBoard {
    pub entity: Entity,
    pub piece: game::Piece,
    pub translation: Vec3,
}

#[derive(Event)]
pub struct EventPlayerMove;

// ---
// Animations
// ---

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct TransformPositionWithYJumpLens {
    pub start: Vec3,
    pub end: Vec3,
}

impl Lens<Transform> for TransformPositionWithYJumpLens {
    fn lerp(&mut self, target: &mut Transform, ratio: f32) {
        let mut value = self.start + (self.end - self.start) * ratio;
        if ratio < 0.5 {
            value.y = ratio * 2.0 + 0.1;
        } else {
            value.y = (1.0 - ratio) * 2.0 + 0.1;
        }
        target.translation = value;
    }
}

// ---
// Systems
// ---

fn player_turn(
    app_state: Res<State<AppState>>,
    game_mode: Res<GameMode>,
    initial_player: Res<InitialPlayer>,
    mut next_state: ResMut<NextState<AppState>>,
    mut game: ResMut<game::Game>,
    // bevy game entities
    mut selected_square: ResMut<SelectedSquare>,
    mut selected_piece: ResMut<SelectedPiece>,
    // queries
    square_query: Query<(Entity, &game::Square)>,
    pieces_query: Query<(Entity, &game::Piece)>,
    mut selections: Query<&mut PickSelection>,
    // events
    mut ew_event_player_move: EventWriter<EventPlayerMove>,
) {
    if *game_mode == GameMode::VsAI && *app_state == AppState::Player2Turn {
        return;
    }

    if *game_mode == GameMode::VsNetwork {
        let initial_player = initial_player.into_inner().0;

        if *app_state == AppState::Player1Turn && initial_player == Player::Two {
            return;
        }
        if *app_state == AppState::Player2Turn && initial_player == Player::One {
            return;
        }
    }

    let (_, new_square) = find_square_by_entity(selected_square.entity, &square_query);

    if new_square.is_none() {
        return;
    }

    let (new_entity, new_piece) = find_piece_by_square(new_square.unwrap(), &pieces_query);
    let (_old_entity, old_piece) = find_piece_by_entity(selected_piece.entity, &pieces_query);

    // Nothing has been selected before
    if selected_piece.entity.is_none()
        && new_entity.is_some()
        && new_piece.unwrap().color == game.state.turn.color
    {
        selected_piece.entity = new_entity;
        return;
    }

    if old_piece.is_none() {
        return;
    }

    let (move_type, state, _) = game.step(old_piece.unwrap(), new_square.unwrap());
    game.state = state.clone();

    // Check whether game move was valid
    match move_type {
        game::MoveType::Invalid => {
            // chain move allowed for same piece
            if new_piece.is_some()
                && new_piece.unwrap().color == game.state.turn.color
                && game.state.turn.chain_count == 0
            {
                selected_piece.entity = new_entity;
            }
        }
        game::MoveType::Regular | game::MoveType::Pass => {
            for mut s in selections.iter_mut() {
                s.is_selected = false;
            }
            selected_piece.deselect();
            selected_square.deselect();

            if *game_mode == GameMode::VsNetwork {
                if *app_state == AppState::Player1Turn {
                    next_state.set(AppState::Player2Turn);
                }
                if *app_state == AppState::Player2Turn {
                    next_state.set(AppState::Player1Turn);
                }
            }

            ew_event_player_move.send(EventPlayerMove);
        }
        _ => {}
    }
}

fn update_entity_pieces(
    mut commands: Commands,
    game: Res<game::Game>,
    // events
    mut event_piece_move: EventWriter<EventPieceMove>,
    mut event_piece_off_board: EventWriter<EventPieceOffBoard>,
    // queries
    mut query: Query<(Entity, &mut Visibility, &mut Transform, &game::Piece)>,
) {
    for (e, mut _v, mut t, p) in query.iter_mut() {
        if !game.is_changed() {
            return;
        }

        let new_piece = game.state.pieces.iter().find(|_p| _p.id == p.id);

        if new_piece.is_none() {
            event_piece_off_board.send(EventPieceOffBoard {
                entity: e,
                piece: *p,
                translation: t.translation,
            });

            commands.entity(e).remove::<game::Piece>();

            continue;
        }

        let new_piece = new_piece.unwrap();

        if p.x != new_piece.x || p.y != new_piece.y {
            commands.entity(e).insert(*new_piece);
            event_piece_move.send(EventPieceMove(e));
        }

        if p.piece_type != new_piece.piece_type {
            t.rotate(Quat::from_rotation_z(-std::f32::consts::PI));
        }
    }
}

fn check_game_termination(game: Res<game::Game>, mut _event_app_exit: ResMut<Events<AppExit>>) {
    match game.check_termination() {
        game::GameTermination::Black(_) => {
            println!("Black won! Thanks for playing!");
            // event_app_exit.send(AppExit);
        }
        game::GameTermination::White(_) => {
            println!("White won! Thanks for playing!");
            // event_app_exit.send(AppExit);
        }
        _ => {}
    }
}

pub fn create_pieces(
    mut commands: Commands,
    game: Res<game::Game>,
    asset_server: Res<AssetServer>,
    square_materials: Res<Materials>,
) {
    let cp_handle = asset_server.load("microsoft.glb#Mesh0/Primitive0");

    for piece in game.state.pieces.iter() {
        let bundle = PbrBundle {
            mesh: cp_handle.clone(),
            material: match piece.color {
                game::Color::Black => square_materials.blue_color.clone(),
                game::Color::White => square_materials.white_color.clone(),
            },
            transform: model_transform(*piece),
            ..Default::default()
        };
        commands.spawn(bundle).insert(*piece);
    }
}

fn highlight_piece(
    selected_piece: Res<SelectedPiece>,
    square_materials: Res<Materials>,
    mut query: Query<(Entity, &game::Piece, &mut Handle<StandardMaterial>)>,
) {
    for (entity, piece, mut material) in query.iter_mut() {
        if Some(entity) == selected_piece.entity {
            *material = square_materials.selected_color.clone();
        } else if piece.color == game::Color::White {
            *material = square_materials.white_color.clone();
        } else {
            *material = square_materials.blue_color.clone();
        }
    }
}

pub fn create_board(
    mut commands: Commands,
    game: Res<game::Game>,
    mut meshes: ResMut<Assets<Mesh>>,
    square_materials: Res<Materials>,
) {
    // Add meshes and materials
    let mesh = meshes.add(Mesh::from(shape::Plane {
        size: 1.,
        ..default()
    }));
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

        commands.spawn((
            bundle,
            PickableBundle::default(),
            // RaycastPickTarget::default(),
            *square,
            On::<Pointer<Click>>::run(event_selected_square),
        ));
    }
}

// --------------
// Event Handlers
// --------------

fn event_selected_square(
    event: Listener<Pointer<Click>>,
    mut selected_square: ResMut<SelectedSquare>,
) {
    selected_square.entity = Some(event.target);
}

fn event_piece_moved(
    mut commands: Commands,
    mut picking_events: EventReader<EventPieceMove>,
    mut query: Query<(Entity, &game::Piece, &Transform)>,
) {
    for event in picking_events.read() {
        let (entity, piece, transform) = query.get_mut(event.0).unwrap();

        let tween = Tween::new(
            EaseFunction::QuadraticInOut,
            Duration::from_millis(1000),
            TransformPositionWithYJumpLens {
                start: transform.translation,
                end: piece_translation(*piece),
            },
        );

        commands.entity(entity).insert(Animator::new(tween));
    }
}

fn event_piece_off_board(
    game: Res<game::Game>,
    mut commands: Commands,
    mut picking_events: EventReader<EventPieceOffBoard>,
) {
    for event in picking_events.read() {
        // let (entity, piece, transform) = query.get_mut(event.0).unwrap();

        let num_removed_pieces = game
            .state
            .removed_pieces
            .iter()
            .filter(|rp| rp.color == event.piece.color)
            .count() as f32;

        let black_start = (0.0, -1.0);
        let white_start = (0.0, 8.0);
        let (start_x, start_y) = match event.piece.color {
            game::Color::Black => (black_start.0 as f32, black_start.1 as f32),
            game::Color::White => (white_start.0 as f32, white_start.1 as f32),
        };

        let translation_end = Vec3::new(start_y, 0.0, start_x + num_removed_pieces - 1.0);

        let tween = Tween::new(
            EaseFunction::QuadraticInOut,
            Duration::from_millis(1000),
            TransformPositionWithYJumpLens {
                start: event.translation,
                end: translation_end,
            },
        );

        commands.entity(event.entity).insert(Animator::new(tween));
    }
}

// ---
// Helpers
// ---

fn find_square_by_entity(
    entity: Option<Entity>,
    square_query: &Query<(Entity, &game::Square)>,
) -> (Option<Entity>, Option<game::Square>) {
    if entity.is_none() {
        return (None, None);
    }

    match square_query.iter().find(|(e, _)| e == &entity.unwrap()) {
        // Square  hold piece
        Some((e, p)) => (Some(e), Some(*p)),

        // Square doesn't hold piece
        _ => (None, None),
    }
}

fn find_piece_by_square(
    square: game::Square,
    pieces_query: &Query<(Entity, &game::Piece)>,
) -> (Option<Entity>, Option<game::Piece>) {
    match pieces_query
        .iter()
        .find(|(_, p)| p.x == square.x && p.y == square.y)
    {
        // Square  hold piece
        Some((e, p)) => (Some(e), Some(*p)),

        // Square doesn't hold piece
        _ => (None, None),
    }
}

fn find_piece_by_entity(
    entity: Option<Entity>,
    pieces_query: &Query<(Entity, &game::Piece)>,
) -> (Option<Entity>, Option<game::Piece>) {
    if entity.is_none() {
        return (None, None);
    }

    match pieces_query.iter().find(|(e, _)| e == &entity.unwrap()) {
        // Square  hold piece
        Some((e, p)) => (Some(e), Some(*p)),

        // Square doesn't hold piece
        _ => (None, None),
    }
}

pub fn piece_translation(piece: game::Piece) -> Vec3 {
    Vec3::new(piece.x as f32, 0.1, piece.y as f32)
}

pub fn model_transform(piece: game::Piece) -> Transform {
    // Translation
    let mut transform = Transform::from_translation(piece_translation(piece));

    // Rotation
    if piece.piece_type == game::PieceType::Normal {
        transform.rotate(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2));
    } else {
        transform.rotate(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2));
    }
    if piece.color == game::Color::Black {
        transform.rotate(Quat::from_rotation_y(-std::f32::consts::FRAC_PI_2));
    } else {
        transform.rotate(Quat::from_rotation_y(std::f32::consts::FRAC_PI_2));
    }
    transform.rotate(Quat::from_rotation_z(-std::f32::consts::PI));

    // Scale
    transform.scale = Vec3::new(0.02, 0.02, 0.02);

    transform
}

// ---
// Plugin
// ---

pub struct BoardPlugin;
impl Plugin for BoardPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SelectedSquare>()
            .init_resource::<SelectedPiece>()
            .add_systems(Startup, create_board)
            .add_systems(
                Update,
                (
                    player_turn,
                    update_entity_pieces.after(player_turn),
                    check_game_termination,
                ),
            )
            .add_event::<EventPieceMove>()
            .add_event::<EventPieceOffBoard>()
            .add_event::<EventPlayerMove>();

        app.add_systems(Startup, create_pieces)
            .add_plugins(TweeningPlugin)
            .add_systems(
                Update,
                (highlight_piece, event_piece_moved, event_piece_off_board).run_if(
                    |app_state: Res<State<AppState>>| {
                        matches!(
                            app_state.into_inner().get(),
                            AppState::Player1Turn | AppState::Player2Turn
                        )
                    },
                ),
            );

        app.insert_resource(InitialPlayer(Player::One));
    }
}
