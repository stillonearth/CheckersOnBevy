use std::sync::Arc;
use std::sync::Mutex;

use checkers_ai::brain;
use checkers_core::game;

mod bevy_frontend;
use bevy_tasks::{TaskPool, TaskPoolBuilder};

fn main() {
    let brain = Arc::new(Mutex::<brain::Brain>::new(brain::Brain::new()));
    let pool = TaskPoolBuilder::new()
        .thread_name("Busy Behavior ThreadPool".to_string())
        .num_threads(1)
        .build();

    let mut app = bevy_frontend::create_bevy_app(game::Game::new());
    app.insert_resource(brain);
    app.insert_resource(pool);
    app.add_plugin(bevy_frontend::UIPlugin);
    app.run();
}
