use bevy::prelude::*;

use environment::environment_client::EnvironmentClient;
use environment::{CurrentStateRequest, JsonReply, ResetRequest, StepRequest};
use serde_json;

use crate::environment::environment_server::Environment;
use futures::executor;

mod animations;
mod bevy_app;
mod board;
mod game;
mod gym_env;
mod materials;
mod ui;

pub mod environment {
    tonic::include_proto!("environment");
}

struct StateUpdateTimer(Timer);

fn fetch_game_state(
    client: &mut EnvironmentClient<tonic::transport::channel::Channel>,
) -> game::GameState {
    let response = client.current_state(CurrentStateRequest {});
    let result = executor::block_on(response).unwrap();
    let state: game::GameState = serde_json::from_str(&result.get_ref().json).unwrap();
    state
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = EnvironmentClient::connect("http://[::1]:50051").await?;

    let sync_game_state = move |mut selected_square: ResMut<board::SelectedSquare>,
                                mut selected_piece: ResMut<board::SelectedPiece>,
                                time: Res<Time>,
                                mut timer: ResMut<StateUpdateTimer>,
                                mut game: ResMut<game::Game>| {
        if game.is_changed() {
            // push state to server
            let reset_request = tonic::Request::new(ResetRequest {
                state: serde_json::to_string(&game.state).unwrap(),
            });

            let _response = executor::block_on(client.reset(reset_request)).unwrap();
        } else if timer.0.tick(time.delta()).just_finished() {
            // pull state from server
            let state = fetch_game_state(&mut client);
            if game.state != state {
                game.state = state;
                game.set_changed();
                selected_piece.deselect();
                selected_square.deselect();
            }
        }
    };

    let game = game::Game::new();
    let mut app = bevy_app::create_bevy_app(game);

    app.insert_resource(StateUpdateTimer(Timer::from_seconds(1.0, true)));
    app.add_system(sync_game_state);
    app.run();

    Ok(())
}
