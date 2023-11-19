use bevy::prelude::*;
use bevy_mod_picking::prelude::Pickable;
use bevy_veilid::*;
use copypasta::*;
use serde::{Deserialize, Serialize};

use crate::board::*;
use crate::*;

use checkers_core::game;

pub struct P2PGamePlugin;

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct CheckersP2PMessage {
    pub game_state: Option<game::GameState>,
    pub extra: Option<String>,
}

#[derive(Resource)]
pub struct P2POverlayUISettings {
    pub awaiting_other_peer: bool,
}

const NORMAL_BUTTON: Color = Color::rgb(0.15, 0.15, 0.35);
const HOVERED_BUTTON: Color = Color::RED;
const _PRESSED_BUTTON: Color = Color::rgb(0.35, 0.75, 0.35);

#[derive(Component)]
struct VeilidDHTKeyText;

#[derive(Component)]
struct VeilidUINode;

#[derive(Component)]
struct VeilidOverlayUINode;

#[derive(Component)]
struct VeilidButtonCopyDHT;

#[derive(Component)]
struct VeilidButtonPasteDHT;

fn switch_veilid_overlay(
    mut query: Query<(&mut Style, &VeilidOverlayUINode)>,
    plugin_settings: Res<P2POverlayUISettings>,
) {
    for (mut s, _) in query.iter_mut() {
        s.display = match plugin_settings.awaiting_other_peer {
            true => Display::Flex,
            false => Display::None,
        };
    }
}

fn on_veilid_initialized(
    mut er_world_initialized: EventReader<EventVeilidInitialized>,
    mut text_query: Query<(&mut Text, &VeilidDHTKeyText)>,
    mut set: ParamSet<(
        Query<(Entity, &VeilidButtonCopyDHT, &mut Visibility)>,
        Query<(Entity, &VeilidButtonPasteDHT, &mut Visibility)>,
    )>,
) {
    for _ in er_world_initialized.read() {
        for (mut text, _tag) in text_query.iter_mut() {
            let str = "Veilid Initialized".to_string();
            text.sections[0].value = str;
        }

        for (_, _, mut v) in set.p0().iter_mut() {
            *v = Visibility::Visible;
        }
        for (_, _, mut v) in set.p1().iter_mut() {
            *v = Visibility::Visible;
        }
    }
}

fn init_veilid_overlay_ui(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    // mut next_state: ResMut<NextState<AppState>>,
) {
    let font = asset_server.load("Roboto-Regular.ttf");
    let text = Text::from_section(
        "Awaiting other player...",
        TextStyle {
            font_size: 35.0,
            font: font.clone(),
            color: Color::WHITE,
        },
    )
    .with_alignment(TextAlignment::Center);

    // root node
    commands
        .spawn((
            NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    width: Val::Percent(100.),
                    height: Val::Percent(20.),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    display: Display::None,
                    ..Default::default()
                },
                z_index: ZIndex::Local(2),
                ..Default::default()
            },
            VeilidOverlayUINode,
        ))
        .with_children(|parent| {
            parent.spawn(TextBundle {
                text,
                ..Default::default()
            });
        })
        .insert(Pickable::IGNORE)
        .insert(Name::new("veilid overlay ui"));
}

fn init_veilid_ui(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    next_state.set(AppState::MainMenu);

    let font = asset_server.load("Roboto-Regular.ttf");
    let text = Text::from_section(
        "INITIALIZING VEILID...",
        TextStyle {
            font_size: 35.0,
            font: font.clone(),
            color: Color::WHITE,
        },
    )
    .with_alignment(TextAlignment::Center);

    // root node
    commands
        .spawn((
            NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    width: Val::Percent(100.),
                    height: Val::Percent(100.),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..Default::default()
                },
                background_color: BackgroundColor(Color::DARK_GRAY),
                z_index: ZIndex::Local(1),
                ..Default::default()
            },
            VeilidUINode,
        ))
        .with_children(|parent| {
            parent
                .spawn(TextBundle {
                    text,
                    ..Default::default()
                })
                .insert(VeilidDHTKeyText);

            parent
                .spawn(NodeBundle {
                    style: Style {
                        flex_direction: FlexDirection::Column,
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .with_children(|parent| {
                    parent
                        .spawn(ButtonBundle {
                            style: Style {
                                width: Val::Px(450.0),
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
                                    "Copy DHT key to clipboard and start game",
                                    TextStyle {
                                        font: font.clone(),
                                        font_size: 25.0,
                                        color: Color::rgb(0.9, 0.9, 0.9),
                                    },
                                ),
                                ..Default::default()
                            });
                        })
                        .insert(Pickable::IGNORE)
                        .insert(VeilidButtonCopyDHT)
                        .insert(Visibility::Hidden);

                    parent
                        .spawn(ButtonBundle {
                            style: Style {
                                width: Val::Px(450.0),
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
                                    "Paste DHT key from clipboard and join game",
                                    TextStyle {
                                        font: font.clone(),
                                        font_size: 25.0,
                                        color: Color::rgb(0.9, 0.9, 0.9),
                                    },
                                ),
                                ..Default::default()
                            });
                        })
                        .insert(Pickable::IGNORE)
                        .insert(VeilidButtonPasteDHT)
                        .insert(Visibility::Hidden);
                });
        })
        .insert(Pickable::IGNORE)
        .insert(Name::new("veilid ui"));
}

#[allow(clippy::type_complexity)]
fn copy_dht_key_button_system(
    mut plugin_settings: ResMut<P2POverlayUISettings>,
    mut initial_player: ResMut<InitialPlayer>,
    mut next_state: ResMut<NextState<AppState>>,
    mut interaction_query: Query<
        (&VeilidButtonCopyDHT, &Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
    mut veilid_ui_query: Query<(&mut Style, With<VeilidUINode>)>,
    veilid_app: Res<VeilidApp>,
) {
    for (_, interaction, mut color) in interaction_query.iter_mut() {
        let veilid_app = veilid_app.app.clone();
        match *interaction {
            Interaction::Pressed => {
                let mut ctx = ClipboardContext::new().unwrap();
                let msg = format!("{}", veilid_app.unwrap().our_dht_key);
                ctx.set_contents(msg.to_owned()).unwrap();
                ctx.get_contents().unwrap();

                for (mut s, _) in veilid_ui_query.iter_mut() {
                    s.display = Display::None;
                }
                next_state.set(AppState::Player1Turn);
                *initial_player = InitialPlayer(Player::One);
                plugin_settings.awaiting_other_peer = true;
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

#[allow(clippy::type_complexity)]
fn paste_dht_key_button_system(
    mut plugin_settings: ResMut<P2POverlayUISettings>,
    mut next_state: ResMut<NextState<AppState>>,
    mut interaction_query: Query<
        (&VeilidButtonPasteDHT, &Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
    mut veilid_ui_query: Query<(&mut Style, With<VeilidUINode>)>,
    veilid_app: Res<VeilidApp>,
    // runtime: ResMut<TokioTasksRuntime>,
    mut ew_send_message: EventWriter<EventSendMessage<CheckersP2PMessage>>,
    mut text_query: Query<(&mut Text, &VeilidDHTKeyText)>,
    mut initial_player: ResMut<InitialPlayer>,
) {
    if veilid_app.app.is_none() {
        return;
    }

    for (_, interaction, mut color) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                // let mut clippy_ctx = ClipboardContext::new().unwrap();
                // let dht_key = clippy_ctx.get_contents().unwrap();
                let dht_key = "".to_string();
                let dht_key = bevy_veilid::veilid_duplex::utils::crypto_key_from_str(dht_key);
                if dht_key.is_err() {
                    for (mut text, _tag) in text_query.iter_mut() {
                        let str = "Bad DHT Key in clipboard, try again".to_string();
                        text.sections[0].value = str;
                    }
                    return;
                }

                ew_send_message.send(EventSendMessage::new(
                    CheckersP2PMessage {
                        game_state: None,
                        extra: Some(String::from("CONNECT")),
                    },
                    dht_key.unwrap(),
                ));

                for (mut s, _) in veilid_ui_query.iter_mut() {
                    s.display = Display::None;
                }
                next_state.set(AppState::Player1Turn);
                *initial_player = InitialPlayer(Player::Two);
                plugin_settings.awaiting_other_peer = true;
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

fn event_on_veilid_message(
    mut er_rm: EventReader<EventReceiveMessage<CheckersP2PMessage>>,
    mut plugin_settings: ResMut<P2POverlayUISettings>,
    app_state: ResMut<State<AppState>>,
    mut next_state: ResMut<NextState<AppState>>,
    mut game: ResMut<game::Game>,
    mut ew_connected_peer: EventWriter<EventConnectedPeer>,
) {
    let current_turn = app_state.into_inner().clone();
    for m in er_rm.read() {
        let message = m.message.clone();
        if let Some(message) = message.extra {
            if message == "CONNECT" {
                plugin_settings.awaiting_other_peer = false;
                ew_connected_peer.send(EventConnectedPeer { dht_key: m.dht_key });
            }
        }

        if let Some(remote_game_state) = message.game_state {
            game.state = remote_game_state;
            let next_turn = match current_turn {
                AppState::Player1Turn => AppState::Player2Turn,
                AppState::Player2Turn => AppState::Player1Turn,
                AppState::MainMenu => AppState::MainMenu,
            };
            next_state.set(next_turn);
        }
    }
}

pub fn send_state_to_peer(
    mut er_player_move: EventReader<EventPlayerMove>,
    mut ew_send_message: EventWriter<EventSendMessage<CheckersP2PMessage>>,
    game: ResMut<game::Game>,
    veilid_add: Res<VeilidApp>,
) {
    for _ in er_player_move.read() {
        println!(
            "sending state {:?} to peer {:?}",
            game.state.clone(),
            veilid_add.other_peer_dht.unwrap()
        );

        ew_send_message.send(EventSendMessage::new(
            CheckersP2PMessage {
                game_state: Some(game.state.clone()),
                extra: None,
            },
            veilid_add.other_peer_dht.unwrap(),
        ));
    }
}

fn on_ev_connected_peer(
    mut er_awaiting_peer: EventReader<EventConnectedPeer>,
    mut plugin_settings: ResMut<P2POverlayUISettings>,
) {
    info!("im here");
    for _ in er_awaiting_peer.read() {
        plugin_settings.awaiting_other_peer = false;
    }
}

impl Plugin for P2PGamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(VeilidPlugin::<CheckersP2PMessage>::default())
            .add_systems(
                Update,
                (
                    on_veilid_initialized,
                    (copy_dht_key_button_system, paste_dht_key_button_system)
                        .run_if(|app_state: Res<State<AppState>>| *app_state == AppState::MainMenu),
                    switch_veilid_overlay,
                    event_on_veilid_message,
                    send_state_to_peer,
                    on_ev_connected_peer,
                ),
            )
            .add_systems(Startup, (init_veilid_ui, init_veilid_overlay_ui))
            .insert_resource(P2POverlayUISettings {
                awaiting_other_peer: false,
            });
    }
}
