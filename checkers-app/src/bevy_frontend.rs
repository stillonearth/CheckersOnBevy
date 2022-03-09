use std::sync::Arc;
use std::sync::Mutex;

use bevy::{app::AppExit, ecs::event::Events, pbr::*, prelude::*, utils::Duration};

use bevy_inspector_egui::WorldInspectorPlugin;
use bevy_mod_picking::*;
use bevy_tasks::TaskPool;
use bevy_tweening::*;

use checkers_ai::brain;
use checkers_core::game;

// ---
// Global Variables
// ---

const DEBUG: bool = true;
const NORMAL_BUTTON: Color = Color::rgb(0.15, 0.15, 0.35);
const HOVERED_BUTTON: Color = Color::rgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::rgb(0.35, 0.75, 0.35);

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

// ---
// States
// ---

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum AppState {
    PlayerTurn,
    ComputerTurn,
}

// ---
// Components
// ---

#[derive(Component)]
struct NextMoveText;

// ---
// Events
// ---

pub struct EventPieceMove(pub Entity);

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

fn player_turn(
    mut app_state: ResMut<State<AppState>>,
    mut game: ResMut<game::Game>,
    // bevy game entities
    mut selected_square: ResMut<SelectedSquare>,
    mut selected_piece: ResMut<SelectedPiece>,
    // queries
    square_query: Query<(Entity, &game::Square)>,
    pieces_query: Query<(Entity, &game::Piece)>,
    mut selections: Query<&mut Selection>,
) {
    if *app_state.current() != AppState::PlayerTurn {
        return;
    }

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

    if old_piece.is_none() {
        return;
    }

    let (move_type, state, _) = game.step(old_piece.unwrap(), new_square.unwrap());
    game.state = state.clone();

    // Check whether game move was valid
    match move_type {
        game::MoveType::Invalid => {
            // chain move allowed for same piece
            if new_piece != None
                && new_piece.unwrap().color == game.state.turn.color
                && game.state.turn.chain_count == 0
            {
                selected_piece.entity = new_entity;
            }
        }
        game::MoveType::Regular | game::MoveType::Pass => {
            for mut s in selections.iter_mut() {
                s.set_selected(false);
            }
            selected_piece.deselect();
            selected_square.deselect();

            if !DEBUG {
                app_state.set(AppState::ComputerTurn).unwrap();
            }
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

fn check_game_termination(game: Res<game::Game>, mut _event_app_exit: ResMut<Events<AppExit>>) {
    match game.check_termination() {
        game::GameTermination::Black => {
            println!("Black won! Thanks for playing!");
            // event_app_exit.send(AppExit);
        }
        game::GameTermination::White => {
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
    // queries
    mut query: Query<(Entity, &mut Transform, &game::Piece)>,
) {
    for (e, mut t, p) in query.iter_mut() {
        if !game.is_changed() {
            return;
        }

        let new_piece = game.state.pieces.iter().find(|_p| _p.id == p.id);

        if new_piece.is_none() {
            commands.entity(e).despawn();
            continue;
        }

        let new_piece = new_piece.unwrap();

        if p.x != new_piece.x || p.y != new_piece.y {
            commands.entity(e).insert(*new_piece);
            event_piece_move.send(EventPieceMove(e));
        }

        if p.piece_type != new_piece.piece_type {
            t.rotate(Quat::from_rotation_z(-2.0 * 1.57));
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
        commands.spawn_bundle(bundle).insert(*piece);
    }
}

pub fn model_transform(piece: game::Piece) -> Transform {
    // Translation
    let mut transform = Transform::from_translation(piece_translation(piece));

    // Rotation
    if piece.piece_type == game::PieceType::Normal {
        transform.rotate(Quat::from_rotation_x(1.57));
    } else {
        transform.rotate(Quat::from_rotation_x(-1.57));
    }
    if piece.color == game::Color::Black {
        transform.rotate(Quat::from_rotation_y(-1.57));
    } else {
        transform.rotate(Quat::from_rotation_y(1.57));
    }
    transform.rotate(Quat::from_rotation_z(-3.14));

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
            TransformPositionWithYJumpLens {
                start: transform.translation,
                end: piece_translation(*piece),
            },
        ));
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
    let text = Text::with_section(
        "",
        TextStyle {
            font_size: 35.0,
            font: asset_server.load("Roboto-Regular.ttf"),
            color: Color::rgb(1.0, 0.2, 0.2),
        },
        TextAlignment {
            horizontal: HorizontalAlign::Left,
            ..Default::default()
        },
    );

    commands.spawn_bundle(UiCameraBundle::default());
    // root node
    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                position: Rect {
                    left: Val::Px(10.),
                    top: Val::Px(10.),
                    ..Default::default()
                },
                ..Default::default()
            },
            visibility: Visibility { is_visible: false },
            ..Default::default()
        })
        .with_children(|parent| {
            parent
                .spawn_bundle(TextBundle {
                    text: text,
                    ..Default::default()
                })
                .insert(NextMoveText);
        });
}

fn init_buttons(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn_bundle(ButtonBundle {
            style: Style {
                size: Size::new(Val::Px(170.0), Val::Px(65.0)),
                // center button
                // margin: Rect::all(Val::Auto),
                // horizontally center child text
                justify_content: JustifyContent::Center,
                // vertically center child text
                align_items: AlignItems::Center,
                ..Default::default()
            },
            color: NORMAL_BUTTON.into(),
            ..Default::default()
        })
        .with_children(|parent| {
            parent.spawn_bundle(TextBundle {
                text: Text::with_section(
                    "Pass Turn",
                    TextStyle {
                        font: asset_server.load("Roboto-Regular.ttf"),
                        font_size: 30.0,
                        color: Color::rgb(0.9, 0.9, 0.9),
                    },
                    Default::default(),
                ),
                ..Default::default()
            });
        });
}

fn button_system(
    mut app_state: ResMut<State<AppState>>,
    mut game: ResMut<game::Game>,
    mut selected_square: ResMut<SelectedSquare>,
    mut selected_piece: ResMut<SelectedPiece>,
    mut interaction_query: Query<
        (&Interaction, &mut UiColor),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, mut color) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Clicked => {
                *color = PRESSED_BUTTON.into();
                selected_square.entity = None;
                selected_piece.entity = None;
                game.state.turn.change();

                if !DEBUG {
                    app_state.set(AppState::ComputerTurn).unwrap();
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
            "Checkers with Rust, Python + AI Agent with AlphaZero\nMove: {}  Turn: {}",
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
    brain: Res<Arc<Mutex<brain::Brain>>>,
    mut app_state: ResMut<State<AppState>>,
    mut game: ResMut<game::Game>,
    task_pool: Res<TaskPool>,
) {
    if *app_state.current() != AppState::ComputerTurn {
        return;
    }

    task_pool.scope(|s| {
        s.spawn(async move {
            let brain = brain.lock().unwrap();
            let mut state = game.state.clone();
            state.moveset = game.possible_moves();
            let action = brain.choose_action(state);
            if action.is_none() {
                game.state.turn.change();
                app_state.set(AppState::PlayerTurn).unwrap();
                // app_state.set_changed();
                return;
            }

            let action = action.unwrap();
            let (move_type, state, _) = game.step(action.piece, action.square);
            game.state = state.clone();
            match move_type {
                game::MoveType::Regular | game::MoveType::Pass => {
                    app_state.set(AppState::PlayerTurn).unwrap();
                }
                game::MoveType::Invalid => {
                    println!("invalid: {:?}", action);
                    // game.state.turn.change();
                    app_state.set(AppState::PlayerTurn).unwrap();
                    return;
                }
                _ => {}
            }
        })
    });
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
            .add_startup_system(create_board)
            .add_system(event_square_selected)
            .add_system(player_turn.label("input"))
            .add_system(update_entity_pieces.after("input"))
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

pub struct UIPlugin;
impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(init_text.system())
            .add_startup_system(init_buttons.system())
            .add_system(next_move_text_update.system())
            .add_system(button_system.system());
    }
}

// ---
// Entry Point
// ---

fn setup(mut commands: Commands) {
    // Light
    commands.spawn_bundle(PointLightBundle {
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
        .spawn_bundle(PerspectiveCameraBundle {
            transform: camera_transform,
            ..Default::default()
        })
        .insert_bundle(PickingCameraBundle::default());
}

pub fn create_bevy_app(game: game::Game) -> App {
    let mut app = App::new();

    app.insert_resource(Msaa { samples: 4 })
        // Set WindowDescriptor Resource to change title and size
        .insert_resource(ClearColor(Color::rgb(0.2, 0.2, 0.2)))
        .insert_resource(WindowDescriptor {
            title: "Checkers!".to_string(),
            width: 800.,
            height: 800.,
            ..Default::default()
        })
        // Resources
        .insert_resource(game)
        // Entry Point
        .add_startup_system(setup.system())
        // External Plugins
        .add_plugins(DefaultPlugins)
        .add_plugin(PickingPlugin)
        .add_plugin(InteractablePickingPlugin)
        // Debug plugins
        // Application Plugins
        .init_resource::<Materials>()
        .add_plugin(BoardPlugin);

    if DEBUG {
        app.add_plugin(WorldInspectorPlugin::new());
    }

    return app;
}
