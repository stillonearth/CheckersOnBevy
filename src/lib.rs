use pyo3::prelude::*;

// mod animations;
// mod bevy_app;
// mod board;
// mod game;
// mod materials;
// mod pieces;
// mod ui;

/// Formats the sum of two numbers as string.
#[pyfunction]
fn run() -> PyResult<()> {
    // let checkers_game_box = Box::new(game::Game::new());
    // let checkers_game: &'static mut game::Game = Box::leak(checkers_game_box);
    // let env = gym_env::CheckersEnv::new(checkers_game);

    Ok(())
}

/// A Python module implemented in Rust. The name of this function must match
/// the `lib.name` setting in the `Cargo.toml`, else Python will not be able to
/// import the module.
#[pymodule]
fn checkers(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(run, m)?)?;
    Ok(())
}
