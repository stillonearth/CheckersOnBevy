use crate::board;
use crate::materials;
use crate::pieces;
use bevy::prelude::*;
use ndarray::prelude::*;

pub struct Observation(arr2);

pub struct Action {
    color: materials::Color,
    piece: pieces::Piece,
    square: board::Square,
}

#[derive(Debug)]
pub struct Step {
    pub obs: Observation,
    pub action: Action,
    pub reward: u8,
    pub is_done: bool,
}

impl Step {
    pub fn make_clone(&self) -> Step {
        Step {
            obs: self.obs.copy(),
            action: self.action,
            reward: self.reward,
            is_done: self.is_done,
        }
    }
}

// An OpenAI Gym session.
pub struct GymEnv {
    app: App,
}

impl GymEnv {
    // Creates a new session of the specified OpenAI Gym environment.
    pub fn new() -> GymEnv {
        GymEnv {}
    }

    pub fn reset(&mut self) -> Observation {}

    // Applies an environment step using the specified action.
    pub fn step(&mut self, action: Action) -> Step {}
}
