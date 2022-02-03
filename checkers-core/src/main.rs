mod bevy_frontend;
mod brain;
pub mod game;
pub mod gym_env;

fn main() {
    let mut app = bevy_frontend::create_bevy_app(game::Game::new());
    let brain = brain::Brain::new();
    app.insert_resource(brain);
    app.add_plugin(bevy_frontend::UIPlugin);
    app.run();
}
