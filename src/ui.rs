use crate::{board::*, game};
use bevy::prelude::*;

use std::sync::Arc;
use std::sync::Mutex;

// Component to mark the Text entity
#[derive(Component)]
struct NextMoveText;

/// Initialize UiCamera and text
fn init_text(mut commands: Commands, asset_server: Res<AssetServer>) {
    let text = Text::with_section(
        "",
        TextStyle {
            font_size: 25.0,
            font: asset_server.load("Roboto-Regular.ttf"),
            color: Color::rgb(0.8, 0.8, 0.8),
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

const NORMAL_BUTTON: Color = Color::rgb(0.15, 0.15, 0.35);
const HOVERED_BUTTON: Color = Color::rgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::rgb(0.35, 0.75, 0.35);

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
fn next_move_text_update(game: Res<game::Game>, mut query: Query<(&mut Text, &NextMoveText)>) {
    let game = game;

    for (mut text, _tag) in query.iter_mut() {
        let str = format!(
            "Checkers with Rust, Python + OpenAI Agent\n\nMove: {}  Turn: {}",
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

pub struct UIPlugin;
impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(init_text.system())
            .add_startup_system(init_buttons.system())
            .add_system(next_move_text_update.system())
            .add_system(button_system.system());
    }
}
