use bevy::prelude::*;
use pyo3::prelude::*;

use std::thread;

use crate::bevy_app;
use crate::game;

use std::sync::Arc;
use std::sync::Mutex;

#[derive(Debug, Clone)]
#[pyclass]
pub struct Action {
    pub piece: game::Piece,
    pub square: game::Square,
}

#[derive(Debug)]
#[pyclass]
pub struct Step {
    #[pyo3(get, set)]
    pub obs: game::GameState,
    #[pyo3(get, set)]
    pub action: Action,
    #[pyo3(get, set)]
    pub reward: i8,
    #[pyo3(get, set)]
    pub is_done: bool,
    #[pyo3(get, set)]
    pub is_valid: bool,
}

// An OpenAI Gym session.
#[pyclass(unsendable)]
pub struct CheckersEnv {
    game: Arc<Mutex<game::Game>>,
    initial_state: game::GameState,
    // app: App,
}

impl CheckersEnv {
    pub fn new(game: Arc<Mutex<game::Game>>) -> CheckersEnv {
        let _game = Arc::clone(&game);
        let initial_state = game.lock().unwrap().state.clone();

        CheckersEnv {
            game,
            initial_state,
            // app: bevy_app::create_bevy_app(_game),
        }
    }
}

#[pymethods]
impl CheckersEnv {
    pub fn start_frontend(&mut self) {
        // self.app.run();
    }

    pub fn current_state(&self) -> game::GameState {
        return self.game.lock().unwrap().state.clone();
    }

    pub fn reset(&mut self) -> game::GameState {
        let mut game = self.game.lock().unwrap();

        game.state = self.initial_state.clone();
        return game.state.clone();
    }

    pub fn step(&mut self, mut action: Action) -> Step {
        let mut game = self.game.lock().unwrap();

        let (move_type, state, termination) = game.step(&mut action.piece, action.square);

        return Step {
            obs: state.clone(),
            action,
            reward: match termination {
                game::GameTermination::Unterminated => 0,
                game::GameTermination::Black => 1,
                game::GameTermination::White => -1,
            },
            is_done: match termination {
                game::GameTermination::Unterminated => false,
                _ => true,
            },
            is_valid: match move_type {
                game::MoveType::Invalid => false,
                _ => true,
            },
        };
    }
}
