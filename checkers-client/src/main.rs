use bevy::prelude::*;

use bevy_tasks::TaskPoolBuilder;
use environment::environment_client::EnvironmentClient;
use environment::{CurrentStateRequest, ResetRequest};
use serde_json;

use futures::executor;

use checkers_app::bevy_frontend::{self, CheckersTaskPool};
use checkers_core::game;

pub mod environment {
    tonic::include_proto!("environment");
}

#[derive(Resource)]
struct StateUpdateTimer(Timer);

#[derive(Resource, Deref, DerefMut)]
struct CheckersGRPCClient(EnvironmentClient<tonic::transport::Channel>);

fn fetch_game_state(client: &mut EnvironmentClient<tonic::transport::Channel>) -> game::GameState {
    let response = client.current_state(CurrentStateRequest {});
    let result = executor::block_on(response).unwrap();
    let state: game::GameState = serde_json::from_str(&result.get_ref().json).unwrap();
    state
}

#[allow(dead_code)]
fn push_game_state(state: game::GameState, client: &mut CheckersGRPCClient) -> game::GameState {
    let reset_request = ResetRequest {
        state: serde_json::to_string(&state).unwrap(),
    };

    let result = executor::block_on(client.reset(reset_request)).unwrap();
    let state: game::GameState = serde_json::from_str(&result.get_ref().json).unwrap();
    state
}

fn sync_game_state(
    mut selected_square: ResMut<bevy_frontend::SelectedSquare>,
    mut selected_piece: ResMut<bevy_frontend::SelectedPiece>,
    time: Res<Time>,
    mut timer: ResMut<StateUpdateTimer>,
    mut game: ResMut<game::Game>,
    mut grpc_client: ResMut<CheckersGRPCClient>,
    task_pool: Res<CheckersTaskPool>,
) {
    task_pool.scope(|s| {
        s.spawn(async move {
            if game.is_changed() {
                // push state to server
                // push_game_state(game.state.clone(), grpc_client.as_mut());
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
    let grpc_client = CheckersGRPCClient(EnvironmentClient::connect("http://[::1]:50051").await?);

    let mut game = game::Game::new();

    let state = fetch_game_state(&mut grpc_client.clone());
    game.state = state;

    let mut app = bevy_frontend::create_bevy_app(game);
    let pool = TaskPoolBuilder::new()
        .thread_name("Busy Behavior ThreadPool".to_string())
        .num_threads(1)
        .build();

    app.insert_resource(grpc_client);
    app.insert_resource(CheckersTaskPool(pool));
    app.add_state(bevy_frontend::AppState::Idle);
    app.insert_resource(StateUpdateTimer(Timer::from_seconds(
        0.01,
        TimerMode::Repeating,
    )));
    app.add_system(sync_game_state);
    app.run();

    Ok(())
}
