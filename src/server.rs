use std::sync::Arc;
use std::sync::Mutex;

use serde_json;
use tonic::{transport::Server, Request, Response, Status};

use environment::environment_server::{Environment, EnvironmentServer};
use environment::{CurrentStateRequest, JsonReply, ResetRequest, StepRequest};

mod game;
mod gym_env;

pub mod environment {
    tonic::include_proto!("environment");
}

pub struct MyEnvironment {
    gym_env: Arc<Mutex<gym_env::CheckersEnv>>,
}

#[tonic::async_trait]
impl Environment for MyEnvironment {
    async fn reset(&self, request: Request<ResetRequest>) -> Result<Response<JsonReply>, Status> {
        let state_json = String::from(&request.into_inner().state);

        let state = match state_json.as_str() {
            "" => None,
            _ => {
                let state: game::GameState = serde_json::from_str(&state_json).unwrap();
                Some(state)
            }
        };

        let mut new_state = self.gym_env.lock().unwrap().reset(state);
        new_state.moveset = self.gym_env.lock().unwrap().game.possible_moves();

        let reply = environment::JsonReply {
            json: serde_json::to_string(&new_state).unwrap(),
        };

        Ok(Response::new(reply))
    }

    async fn step(&self, request: Request<StepRequest>) -> Result<Response<JsonReply>, Status> {
        let action_json = String::from(&request.into_inner().action);
        let action: gym_env::Action = serde_json::from_str(&action_json).unwrap();

        let step = self.gym_env.lock().unwrap().step(action);

        let reply = environment::JsonReply {
            json: serde_json::to_string(&step).unwrap(),
        };

        Ok(Response::new(reply))
    }

    async fn current_state(
        &self,
        _: Request<CurrentStateRequest>,
    ) -> Result<Response<JsonReply>, Status> {
        let mut game_state = self.gym_env.lock().unwrap().game.state.clone();
        game_state.moveset = self.gym_env.lock().unwrap().game.possible_moves();

        let reply = environment::JsonReply {
            json: serde_json::to_string(&game_state).unwrap(),
        };

        Ok(Response::new(reply))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let env = MyEnvironment {
        gym_env: Arc::new(Mutex::<gym_env::CheckersEnv>::new(
            gym_env::CheckersEnv::new(game::Game::new()),
        )),
    };

    let addr = "[::1]:50051".parse()?;

    Server::builder()
        .add_service(EnvironmentServer::new(env))
        .serve(addr)
        .await?;

    Ok(())
}