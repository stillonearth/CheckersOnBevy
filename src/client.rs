use bevy::prelude::*;

use bevy_tasks::{TaskPool, TaskPoolBuilder};
use environment::environment_client::EnvironmentClient;
use environment::{CurrentStateRequest, ResetRequest};
use serde_json;

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

fn push_game_state(
    state: game::GameState,
    client: &mut EnvironmentClient<tonic::transport::channel::Channel>,
) -> game::GameState {
    let reset_request = ResetRequest {
        state: serde_json::to_string(&state).unwrap(),
    };

    let result = executor::block_on(client.reset(reset_request)).unwrap();
    let state: game::GameState = serde_json::from_str(&result.get_ref().json).unwrap();
    state
}

fn sync_game_state(
    mut selected_square: ResMut<board::SelectedSquare>,
    mut selected_piece: ResMut<board::SelectedPiece>,
    time: Res<Time>,
    mut timer: ResMut<StateUpdateTimer>,
    mut game: ResMut<game::Game>,
    mut grpc_client: ResMut<EnvironmentClient<tonic::transport::channel::Channel>>,
    task_pool: Res<TaskPool>,
) {
    task_pool.scope(|s| {
        s.spawn(async move {
            if game.is_changed() {
                // push state to server
                push_game_state(game.state.clone(), grpc_client.as_mut());
            } else if timer.0.tick(time.delta()).just_finished() {
                // pull state from server
                let state = fetch_game_state(grpc_client.as_mut());
                if game.state.pieces != state.pieces {
                    selected_piece.deselect();
                    selected_square.deselect();

                    game.state = state;
                    game.set_changed();
                }
            }
        })
    });
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let grpc_client = EnvironmentClient::connect("http://[::1]:50051").await?;

    let mut game = game::Game::new();

    let state = fetch_game_state(&mut grpc_client.clone());
    game.state = state;

    let mut app = bevy_app::create_bevy_app(game);
    let pool = TaskPoolBuilder::new()
        .thread_name("Busy Behavior ThreadPool".to_string())
        .num_threads(4)
        .build();

    app.insert_resource(grpc_client);
    app.insert_resource(pool);
    app.insert_resource(StateUpdateTimer(Timer::from_seconds(0.01, true)));
    app.add_system(sync_game_state);
    app.run();

    Ok(())
}
