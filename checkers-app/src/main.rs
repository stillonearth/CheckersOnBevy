use std::sync::Arc;
use std::sync::Mutex;

use bevy_frontend::CheckersBrain;
use bevy_frontend::CheckersTaskPool;
use checkers_ai::brain;
use checkers_core::game;

mod bevy_frontend;
use bevy_tasks::TaskPoolBuilder;

fn main() {
    let root_dir = env!("CARGO_MANIFEST_DIR");
    let model_path = format!("{}{}", root_dir, "/assets/model.onnx");

    let brain = CheckersBrain(Arc::new(Mutex::new(brain::Brain::new(model_path))));
    let pool = CheckersTaskPool(
        TaskPoolBuilder::new()
            .thread_name("Busy Behavior ThreadPool".to_string())
            .num_threads(1)
            .build(),
    );
    let game = game::Game::new();

    let mut app = bevy_frontend::create_bevy_app(game /*pool, brain*/);

    app.insert_resource(brain);
    app.insert_resource(pool);

    app.add_state(bevy_frontend::AppState::PlayerTurn);
    app.add_system(bevy_frontend::computer_turn);
    app.add_plugin(bevy_frontend::UIPlugin);

    app.run();
}
