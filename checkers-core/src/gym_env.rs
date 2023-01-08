use crate::game;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Action {
    pub piece: game::Piece,
    pub square: game::Square,
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
        if let Some(state) = state {
            self.game.state = state;
        } else {
            self.game.state = self.initial_state.clone();
        }

        self.game.state.clone()
    }

    pub fn step(&mut self, action: Action) -> Step {
        let (_move_type, state, termination) = self.game.step(action.piece, action.square);

        Step {
            obs: state.clone(),
            action,
            reward: match termination {
                game::GameTermination::Unterminated => 0,
                game::GameTermination::Black(num) => num as i8,
                game::GameTermination::Draw => 0,
                game::GameTermination::White(num) => -(num as i8),
            },
            is_done: !matches!(termination, game::GameTermination::Unterminated),
        }
    }
}
