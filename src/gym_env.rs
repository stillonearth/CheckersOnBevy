use bevy::prelude::*;

use crate::bevy_app;
use crate::game;

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
    checkers_game: &'static mut game::Game,
    initial_state: game::GameState,
    // bevy_app: App,
}

impl CheckersEnv {
    pub fn new(checkers_game: &'static mut game::Game) -> CheckersEnv {
        let initial_state = checkers_game.state.clone();
        // let app = bevy_app::create_bevy_app(checkers_game);

        CheckersEnv {
            // bevy_app: app,
            checkers_game,
            initial_state,
        }
    }

    pub fn start_bevy_app(&mut self) {
        // self.bevy_app.run();
    }

    pub fn current_state(&self) -> &game::GameState {
        return &self.checkers_game.state;
    }

    pub fn reset(&mut self) -> game::GameState {
        self.checkers_game.state = self.initial_state.clone();
        return self.checkers_game.state.clone();
    }

    // Applies an environment step using the specified action.
    pub fn step(&mut self, mut action: Action) -> Step {
        let (move_type, state, termination) =
            self.checkers_game.step(&mut action.piece, action.square);

        return Step {
            obs: state.clone(),
            action: action,
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
