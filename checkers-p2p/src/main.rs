use checkers_app::app::*;
use checkers_app::*;
use checkers_core::game;

fn main() {
    let mut app = create_bevy_app(game::Game::new(), GameMode::VsNetwork);

    app.run();
}
