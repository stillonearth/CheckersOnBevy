use bevy::prelude::*;
use bevy_mod_picking::prelude::*;

use checkers_core::game;

use crate::board::*;
use crate::*;

const NORMAL_BUTTON: Color = Color::rgb(0.15, 0.15, 0.35);
const HOVERED_BUTTON: Color = Color::rgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::rgb(0.35, 0.75, 0.35);

#[derive(Component)]
struct NextMoveText;

#[derive(Component)]
struct ButtonPassTurn;

fn init_text(mut commands: Commands, asset_server: Res<AssetServer>) {
    let text = Text::from_section(
        "",
        TextStyle {
            font_size: 20.0,
            font: asset_server.load("Roboto-Regular.ttf"),
            color: Color::WHITE,
        },
    )
    .with_alignment(TextAlignment::Left);

    // root node
    commands
        .spawn(NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                left: Val::Px(10.),
                bottom: Val::Px(10.),
                ..Default::default()
            },
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
        .spawn((
            ButtonBundle {
                style: Style {
                    width: Val::Px(170.0),
                    height: Val::Px(65.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..Default::default()
                },
                background_color: NORMAL_BUTTON.into(),
                ..Default::default()
            },
            ButtonPassTurn,
        ))
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

/// Update text with the correct turn
fn next_move_text_update(
    app_state: Res<State<AppState>>,
    game_mode: Res<GameMode>,
    initial_player: Res<InitialPlayer>,
    game: Res<game::Game>,
    mut text_query: Query<(&mut Text, &NextMoveText)>,
) {
    let game_mode = *game_mode.into_inner();

    for (mut text, _tag) in text_query.iter_mut() {
        let str = format!(
            "CheckersOnBevy\nMode: {}\nAppState: {:?}\nPLAYER: {}\nWho's turn: {}\nTurn #: {} ",
            match game_mode {
                GameMode::VsAI => "VS AI",
                GameMode::VsNetwork => "VS NETWORK",
                GameMode::VsPlayer => "2 PLAYER",
            },
            app_state,
            match initial_player.0 {
                Player::One => "WHITE",
                Player::Two => "BLACK",
            },
            match game.state.turn.color {
                game::Color::White => "WHITE",
                game::Color::Black => "BLACK",
            },
            game.state.turn.turn_count
        )
        .to_string();
        text.sections[0].value = str;
    }
}

#[allow(clippy::type_complexity)]
fn pass_turn_button_system(
    game_mode: Res<GameMode>,
    state: Res<State<AppState>>,
    mut next_state: ResMut<NextState<AppState>>,
    mut game: ResMut<game::Game>,
    mut selected_square: ResMut<SelectedSquare>,
    mut selected_piece: ResMut<SelectedPiece>,
    mut interaction_query: Query<
        (&ButtonPassTurn, &Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (_, interaction, mut color) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                *color = PRESSED_BUTTON.into();
                selected_square.entity = None;
                selected_piece.entity = None;
                game.state.turn.change();

                if game_mode.clone() == GameMode::VsNetwork {
                    if *state.get() == AppState::Player1Turn {
                        next_state.set(AppState::Player2Turn);
                    }
                    if *state.get() == AppState::Player2Turn {
                        next_state.set(AppState::Player1Turn);
                    }
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

pub struct UIPlugin;
impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (init_text, init_buttons))
            .add_systems(
                Update,
                (next_move_text_update, pass_turn_button_system).run_if(
                    |app_state: Res<State<AppState>>| {
                        matches!(
                            app_state.into_inner().get(),
                            AppState::Player1Turn | AppState::Player2Turn
                        )
                    },
                ),
            );
    }
}
