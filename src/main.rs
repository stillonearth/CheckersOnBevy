mod bevy_frontend;
mod game;
mod gym_env;

fn main() {
    let mut app = bevy_frontend::create_bevy_app(game::Game::new());
    app.run();
}
