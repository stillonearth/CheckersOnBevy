mod animations;
mod bevy_app;
mod board;
mod game;
mod gym_env;
mod materials;
mod ui;

fn main() {
    let mut app = bevy_app::create_bevy_app(game::Game::new());
    app.run();
}
