use checkers_app::app::*;
use checkers_app::*;
use checkers_core::game;
use clap::Parser;

#[derive(Parser, Debug)]
struct Args {
    #[arg(long)]
    mode: String,
}

fn main() {
    let args = Args::parse();

    let game_mode = match args.mode.as_str() {
        "ai" => GameMode::VsAI,
        "p2p" => GameMode::VsNetwork,
        _ => GameMode::VsPlayer,
    };

    let mut app = create_bevy_app(game::Game::new(), game_mode);
    app.add_state::<AppState>();

    app.run();
}
