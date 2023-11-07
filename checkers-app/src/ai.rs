use std::sync::{Arc, Mutex};

use bevy::prelude::*;

use bevy_tasks::{TaskPool, TaskPoolBuilder};

use crate::*;
use checkers_ai::brain;
use checkers_core::game;

#[derive(Resource, Deref, DerefMut, Debug)]
pub struct CheckersBrain(pub Arc<Mutex<brain::Brain>>);

#[derive(Resource, Deref, DerefMut, Debug)]
pub struct CheckersTaskPool(pub TaskPool);

pub fn ai_turn(
    app_state: ResMut<State<AppState>>,
    game_mode: Res<GameMode>,
    mut next_state: ResMut<NextState<AppState>>,
    mut game: ResMut<game::Game>,
    brain: Res<CheckersBrain>,
    task_pool: Res<CheckersTaskPool>,
) {
    if *game_mode.into_inner() != GameMode::VsAI {
        return;
    }

    if *app_state.into_inner() != AppState::Player2Turn {
        return;
    }

    task_pool.scope(|s| {
        s.spawn(async move {
            let mut state = game.state.clone();
            let brain = brain.lock().unwrap();
            state.moveset = game.possible_moves();
            let action = brain.choose_action(state);
            if action.is_none() {
                game.state.turn.change();
                next_state.set(AppState::Player1Turn);
                return;
            }

            let action = action.unwrap();
            let (move_type, state, _) = game.step(action.piece, action.square);
            game.state = state.clone();
            match move_type {
                game::MoveType::Regular | game::MoveType::Pass => {
                    next_state.set(AppState::Player1Turn);
                }
                game::MoveType::Invalid => {
                    println!("invalid: {:?}", action);
                    next_state.set(AppState::Player1Turn);
                }
                _ => {}
            }
        })
    });
}

pub struct AIGamePlugin;

impl Plugin for AIGamePlugin {
    fn build(&self, app: &mut App) {
        let root_dir = env!("CARGO_MANIFEST_DIR");
        let model_path = format!("{}{}", root_dir, "/assets/model.onnx");

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
}
