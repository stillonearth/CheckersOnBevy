use bevy::pbr::*;
use bevy::prelude::*;
use bevy_inspector_egui::WorldInspectorPlugin;
use bevy_mod_picking::*;

use std::sync::Arc;
use std::sync::Mutex;

mod animations;
mod bevy_app;
mod board;
mod game;
mod gym_env;
mod materials;
mod pieces;
mod ui;

fn main() {
    let game = Arc::new(Mutex::<game::Game>::new(game::Game::new()));

    let mut app = bevy_app::create_bevy_app(Arc::clone(&game));
    app.run();
}
