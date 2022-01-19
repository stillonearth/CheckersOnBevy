use crate::{board::*, materials};
use bevy::prelude::*;

// Component to mark the Text entity
#[derive(Component)]
struct NextMoveText;

/// Initialize UiCamera and text
fn init_next_move_text(mut commands: Commands, asset_server: Res<AssetServer>) {
    let text = Text::with_section(
        "Next move: White",
        TextStyle {
            font_size: 40.0,
            font: asset_server.load("Roboto-Regular.ttf"),
            color: Color::rgb(0.8, 0.8, 0.8),
        },
        TextAlignment {
            horizontal: HorizontalAlign::Center,
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

/// Update text with the correct turn
fn next_move_text_update(turn: ResMut<PlayerTurn>, mut query: Query<(&mut Text, &NextMoveText)>) {
    if turn.is_changed() {
        return;
    }

    for (mut text, _tag) in query.iter_mut() {
        let str = format!(
            "Next move: {}",
            match turn.0 {
                materials::Color::White => "White",
                materials::Color::Black => "Black",
            }
        )
        .to_string();
        text.sections[0].value = str;
    }
}

pub struct UIPlugin;
impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(init_next_move_text.system())
            .add_system(next_move_text_update.system());
    }
}
