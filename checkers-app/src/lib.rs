use bevy::prelude::*;

#[derive(Resource, PartialEq, Eq, Copy, Clone)]
pub enum GameMode {
    VsAI,
    VsPlayer,
    VsNetwork,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, States, Default)]
pub enum AppState {
    #[default]
    Player1Turn,
    Player2Turn,
    MainMenu,
}

pub mod ai;
pub mod app;
pub mod board;

mod ui;
mod veilid;
