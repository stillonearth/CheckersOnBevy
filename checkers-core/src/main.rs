mod bevy_frontend;
pub mod game;
pub mod gym_env;

fn main() {
    let mut app = bevy_frontend::create_bevy_app(game::Game::new());
    app.run();
}
