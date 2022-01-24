use bevy::pbr::*;
use bevy::prelude::*;
use bevy_inspector_egui::WorldInspectorPlugin;
use bevy_mod_picking::*;

mod animations;
mod bevy_app;
mod board;
mod game;
mod materials;
mod pieces;
mod ui;

fn main() {
    let checkers_game_box = Box::new(game::Game::new());
    let checkers_game: &'static mut game::Game = Box::leak(checkers_game_box);

    bevy_app::create_bevy_app(checkers_game).run();
}
