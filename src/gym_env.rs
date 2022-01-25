use bevy::prelude::*;

use crate::game;

use std::sync::Arc;
use std::sync::Mutex;

#[derive(Debug, Clone)]
pub struct Action {
    pub piece: game::Piece,
    pub square: game::Square,
}

#[derive(Debug)]
pub struct Step {
    pub obs: game::GameState,
    pub action: Action,
    pub reward: i8,
    pub is_done: bool,
    pub is_valid: bool,
}

// An OpenAI Gym session.
pub struct CheckersEnv {
    game: Arc<Mutex<game::Game>>,
    initial_state: game::GameState,
    bevy_app: App,
}

impl CheckersEnv {
    pub fn new(game: Arc<Mutex<game::Game>>, app: App) -> CheckersEnv {
        let initial_state = game.lock().unwrap().state.clone();

        CheckersEnv {
            bevy_app: app,
            game,
            initial_state,
        }
    }

    pub fn start_bevy_app(&mut self) {
        self.bevy_app.run();
    }

    pub fn current_state(&self) -> game::GameState {
        return self.game.lock().unwrap().state.clone();
    }

    pub fn reset(&mut self) -> game::GameState {
        let mut game = self.game.lock().unwrap();

        game.state = self.initial_state.clone();
        return game.state.clone();
    }

    // Applies an environment step using the specified action.
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
