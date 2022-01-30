use crate::game;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Action {
    pub piece: Option<game::Piece>,
    pub square: Option<game::Square>,
}

#[derive(Debug, Serialize)]
pub struct Step {
    pub obs: game::GameState,
    pub action: Action,
    pub reward: i8,
    pub is_done: bool,
}

// An OpenAI Gym session.
pub struct CheckersEnv {
    pub game: game::Game,
    initial_state: game::GameState,
}

impl CheckersEnv {
    pub fn new(game: game::Game) -> CheckersEnv {
        let initial_state = game.state.clone();

        CheckersEnv {
            game,
            initial_state,
        }
    }

    pub fn reset(&mut self, state: Option<game::GameState>) -> game::GameState {
        if state == None {
            self.game.state = self.initial_state.clone();
        } else {
            self.game.state = state.unwrap().clone();
        }
        return self.game.state.clone();
    }

    pub fn step(&mut self, mut action: Action) -> Step {
        let (_move_type, state, termination) = self.game.step(action.piece, action.square);

        return Step {
            obs: state.clone(),
            action,
            reward: match termination {
                game::GameTermination::Unterminated => 0,
                game::GameTermination::Black => 10,
                game::GameTermination::Draw => 0,
                game::GameTermination::White => -10,
                game::GameTermination::BlackMoveLimit => 5,
                game::GameTermination::WhiteMoveLimit => -5,
            },
            is_done: match termination {
                game::GameTermination::Unterminated => false,
                _ => true,
            },
        };
    }
}
