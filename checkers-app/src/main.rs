use std::sync::Arc;
use std::sync::Mutex;

use bevy::prelude::*;
use bevy_tasks::TaskPoolBuilder;
use clap::Parser;

use checkers_ai::brain;
use checkers_core::game;

use checkers_app::bevy_frontend::*;

#[derive(Parser, Debug)]
struct Args {
    #[arg(long)]
    mode: String,
}

fn main() {
    let args = Args::parse();

    let root_dir = env!("CARGO_MANIFEST_DIR");
    let model_path = format!("{}{}", root_dir, "/assets/model.onnx");

    let game_mode = match args.mode.as_str() {
        "ai" => GameMode::VsAI,
        "p2p" => GameMode::VsNetwork,
        _ => GameMode::VsPlayer,
    };

    let mut app = create_bevy_app(game::Game::new(), game_mode);

    if game_mode == GameMode::VsAI {
        let brain = CheckersBrain(Arc::new(Mutex::new(brain::Brain::new(model_path))));
        let pool = CheckersTaskPool(
            TaskPoolBuilder::new()
                .thread_name("Busy Behavior ThreadPool".to_string())
                .num_threads(1)
                .build(),
        );

        app.insert_resource(brain);
        app.insert_resource(pool);
        app.add_systems(Update, ai_turn);
    }

    app.add_state::<AppState>();
    app.add_plugins(UIPlugin);

    app.run();
}
