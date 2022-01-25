use pyo3::prelude::*;

use std::sync::Arc;
use std::sync::Mutex;

mod animations;
mod bevy_app;
mod board;
mod game;
mod gym_env;
mod materials;
mod pieces;
mod ui;

/// Formats the sum of two numbers as string.
#[pyfunction]
fn new() -> PyResult<gym_env::CheckersEnv> {
    let game = Arc::new(Mutex::<game::Game>::new(game::Game::new()));
    let env = gym_env::CheckersEnv::new(game);

    Ok(env)
}

/// A Python module implemented in Rust. The name of this function must match
/// the `lib.name` setting in the `Cargo.toml`, else Python will not be able to
/// import the module.
#[pymodule]
fn checkers(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<gym_env::CheckersEnv>()?;
    m.add_function(wrap_pyfunction!(new, m)?)?;
    Ok(())
}
