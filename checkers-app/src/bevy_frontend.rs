use std::sync::{Arc, Mutex};

use bevy::{app::AppExit, ecs::event::Events, pbr::*, prelude::*, utils::Duration};

// use bevy_mod_picking::selection::*;
// use bevy_mod_picking::*;
use bevy_mod_picking::prelude::*;
use bevy_tasks::TaskPool;
use bevy_tweening::*;

use checkers_ai::brain::Brain;
use checkers_core::game::{self, black_start_positions, white_start_positions};

// ---
// Global Variables
// ---

const DEBUG: bool = false;
const NORMAL_BUTTON: Color = Color::rgb(0.15, 0.15, 0.35);
const HOVERED_BUTTON: Color = Color::rgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::rgb(0.35, 0.75, 0.35);

// ---
// Resources
// ---

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

#[derive(Resource, Deref, DerefMut, Debug)]
pub struct CheckersBrain(pub Arc<Mutex<Brain>>);

#[derive(Resource, Deref, DerefMut, Debug)]
pub struct CheckersTaskPool(pub TaskPool);

// ---
// States
// ---

#[derive(Debug, Clone, Eq, PartialEq, Hash, States, Default)]
pub enum AppState {
    #[default]
    PlayerTurn,
    ComputerTurn,
    #[allow(dead_code)]
    Idle,
}

// ---
// Components
// ---

#[derive(Component)]
struct NextMoveText;

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

// ---
// Systems
// ---

// Board & squares

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
            RaycastPickTarget::default(),
            *square,
            On::<Pointer<Click>>::run(event_selected_square),
        ));
    }
}

fn event_selected_square(
    event: Listener<Pointer<Click>>,
    mut selected_square: ResMut<SelectedSquare>,
) {
    selected_square.entity = Some(event.target);
    return;
}

fn player_turn(
    current_state: ResMut<State<AppState>>,
    mut next_state: ResMut<NextState<AppState>>,
    mut game: ResMut<game::Game>,
    // bevy game entities
    mut selected_square: ResMut<SelectedSquare>,
    mut selected_piece: ResMut<SelectedPiece>,
    // queries
    square_query: Query<(Entity, &game::Square)>,
    pieces_query: Query<(Entity, &game::Piece)>,
    mut selections: Query<&mut PickSelection>,
) {
    if *current_state != AppState::PlayerTurn {
        return;
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

            if !DEBUG {
                next_state.set(AppState::ComputerTurn);
            }
        }
        _ => {}
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

// Pieces — Draughts

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

fn event_piece_moved(
    mut commands: Commands,
    mut picking_events: EventReader<EventPieceMove>,
    mut query: Query<(Entity, &game::Piece, &Transform)>,
) {
    for event in picking_events.iter() {
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
    for event in picking_events.iter() {
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

// UI -- Buttons & Text

fn init_text(mut commands: Commands, asset_server: Res<AssetServer>) {
    let text = Text::from_section(
        "",
        TextStyle {
            font_size: 35.0,
            font: asset_server.load("Roboto-Regular.ttf"),
            color: Color::rgb(1.0, 0.2, 0.2),
        },
    )
    .with_alignment(TextAlignment::Left);

    // root node
    commands
        .spawn(NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                left: Val::Px(10.),
                top: Val::Px(10.),
                ..Default::default()
            },
            visibility: Visibility::Hidden,
            ..Default::default()
        })
        .with_children(|parent| {
            parent
                .spawn(TextBundle {
                    text,
                    ..Default::default()
                })
                .insert(NextMoveText);
        })
        .insert(Pickable::IGNORE);
}

fn init_buttons(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn(ButtonBundle {
            style: Style {
                width: Val::Px(170.0),
                height: Val::Px(65.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..Default::default()
            },
            background_color: NORMAL_BUTTON.into(),
            ..Default::default()
        })
        .with_children(|parent| {
            parent.spawn(TextBundle {
                text: Text::from_section(
                    "Pass Turn",
                    TextStyle {
                        font: asset_server.load("Roboto-Regular.ttf"),
                        font_size: 30.0,
                        color: Color::rgb(0.9, 0.9, 0.9),
                    },
                ),
                ..Default::default()
            });
        })
        .insert(Pickable::IGNORE);
}

#[allow(clippy::type_complexity)]
fn button_system(
    mut next_state: ResMut<NextState<AppState>>,
    mut game: ResMut<game::Game>,
    mut selected_square: ResMut<SelectedSquare>,
    mut selected_piece: ResMut<SelectedPiece>,
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, mut color) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                *color = PRESSED_BUTTON.into();
                selected_square.entity = None;
                selected_piece.entity = None;
                game.state.turn.change();

                if !DEBUG {
                    next_state.set(AppState::ComputerTurn);
                }
            }
            Interaction::Hovered => {
                *color = HOVERED_BUTTON.into();
            }
            Interaction::None => {
                *color = NORMAL_BUTTON.into();
            }
        }
    }
}

/// Update text with the correct turn
fn next_move_text_update(game: Res<game::Game>, mut text_query: Query<(&mut Text, &NextMoveText)>) {
    let game = game;

    for (mut text, _tag) in text_query.iter_mut() {
        let str = format!(
            "CheckersOnBevy\nMove: {}  Turn: {}",
            match game.state.turn.color {
                game::Color::White => "White",
                game::Color::Black => "Black",
            },
            game.state.turn.turn_count
        )
        .to_string();
        text.sections[0].value = str;
    }
}

// AI -- moving in game

pub fn computer_turn(
    app_state: ResMut<State<AppState>>,
    mut next_state: ResMut<NextState<AppState>>,
    mut game: ResMut<game::Game>,
    brain: Res<CheckersBrain>,
    task_pool: Res<CheckersTaskPool>,
) {
    if *app_state.into_inner() != AppState::ComputerTurn {
        return;
    }

    task_pool.scope(|s| {
        s.spawn(async move {
            let mut state = game.state.clone();
            let brain = brain.lock().unwrap();
            state.moveset = game.possible_moves();
            let action = brain.choose_action(state);
            if action.is_none() {
                game.state.turn.change();
                next_state.set(AppState::PlayerTurn);
                // app_state.set_changed();
                return;
            }

            let action = action.unwrap();
            let (move_type, state, _) = game.step(action.piece, action.square);
            game.state = state.clone();
            match move_type {
                game::MoveType::Regular | game::MoveType::Pass => {
                    next_state.set(AppState::PlayerTurn);
                }
                game::MoveType::Invalid => {
                    println!("invalid: {:?}", action);
                    // game.state.turn.change();
                    next_state.set(AppState::PlayerTurn);
                }
                _ => {}
            }
        })
    });
}

// ---
// Helpers
// ---

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

pub fn piece_translation(piece: game::Piece) -> Vec3 {
    Vec3::new(piece.x as f32, 0.1, piece.y as f32)
}

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
// Plugins
// ---

pub struct BoardPlugin;
impl Plugin for BoardPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SelectedSquare>()
            .init_resource::<SelectedPiece>()
            .add_systems(Startup, create_board)
            .add_systems(Update, player_turn)
            .add_systems(Update, update_entity_pieces.after(player_turn))
            .add_systems(Update, check_game_termination)
            .add_event::<EventPieceMove>()
            .add_event::<EventPieceOffBoard>()
            .add_plugins(PiecesPlugin);
    }
}

pub struct PiecesPlugin;
impl Plugin for PiecesPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, create_pieces)
            .add_plugins(TweeningPlugin)
            .add_systems(Update, highlight_piece)
            .add_systems(Update, event_piece_moved)
            .add_systems(Update, event_piece_off_board);
    }
}

pub struct UIPlugin;
impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, init_text)
            .add_systems(Startup, init_buttons)
            .add_systems(Update, next_move_text_update)
            .add_systems(Update, button_system);
    }
}

// ---
// Entry Point
// ---

fn setup(mut commands: Commands) {
    // Light
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 3000.0,
            shadows_enabled: false,
            ..Default::default()
        },
        transform: Transform::from_xyz(0.0, 4.0, 3.5),
        ..Default::default()
    });

    let mut camera_transform = Transform::from_matrix(Mat4::from_rotation_translation(
        Quat::from_xyzw(-0.3, -0.5, -0.3, 0.5).normalize(),
        Vec3::new(-7.5, 20.0, 3.5),
    ));

    camera_transform.scale.z = 1.5;

    // Camera
    commands
        .spawn(Camera3dBundle {
            transform: camera_transform,
            ..Default::default()
        })
        .insert(RaycastPickCamera::default());
}

pub fn create_bevy_app(game: game::Game, /*pool: CheckersTaskPool, brain: CheckersBrain*/) -> App {
    let mut app = App::new();

    app.insert_resource(ClearColor(Color::rgb(0.2, 0.2, 0.2)))
        .insert_resource(game)
        // External Plugins
        // .add_plugins(DefaultPlugins.set(WindowPlugin {
        //     window: WindowDescriptor {
        //         title: "Checkers on Bevy!".to_string(),
        //         width: 800.,
        //         height: 800.,
        //         ..default()
        //     },
        //     ..default()
        // }))
        .add_plugins(DefaultPlugins.set(low_latency_window_plugin()))
        .add_plugins(DefaultPickingPlugins)
        .add_systems(Update, bevy_mod_picking::debug::hide_pointer_text)
        .init_resource::<Materials>()
        .add_plugins(BoardPlugin)
        .add_systems(Startup, setup);

    app
}
