use rand;
use rand::{distributions::WeightedIndex, prelude::Distribution};
use tract_ndarray::Array3;
use tract_onnx::prelude::*;

use checkers_core::game;
use checkers_core::gym_env;

#[derive(Debug, Clone)]
pub struct Brain {
    model_path: String,
}

impl Brain {
    pub fn new(model_path: String) -> Brain {
        Brain {
            model_path: model_path,
        }
    }

    pub fn choose_action(&self, state: game::GameState) -> Option<gym_env::Action> {
        let model = tract_onnx::onnx()
            .model_for_path(self.model_path.as_str())
            .unwrap()
            .with_input_fact(0, f32::fact(&[1, 8, 8]).into())
            .unwrap()
            .into_optimized()
            .unwrap()
            .into_runnable()
            .unwrap();

        let zeros: Vec<f32> = (0..64).map(|_| 0.0).collect();
        let mut input_array = Array3::from_shape_vec((1, 8, 8), zeros).unwrap();

        let multiplier = match state.turn.color {
            game::Color::Black => 1,
            game::Color::White => -1,
        };

        for p in state.pieces.iter() {
            let value = (multiplier
                * match p.color {
                    game::Color::Black => 1,
                    game::Color::White => -1,
                }) as f32;

            // play from perspective of black; flip board if white
            let index = [0, 7 - p.x as usize, p.y as usize];
            input_array[index] = value;
        }

        let input: Tensor = input_array.into();
        let result = model.run(tvec!(input)).unwrap();
        let output = result[0].to_array_view::<f32>().unwrap();

        let allowed_moves = state.moveset;
        let mut actions: Vec<(u8, u8, u8, u8)> = Vec::new();
        let mut weights: Vec<i32> = Vec::new();

        for p in &state.pieces {
            for m in allowed_moves[p.id as usize].iter() {
                if p.color != state.turn.color {
                    continue;
                }

                let index = [0, p.x as usize, p.y as usize, m.0 as usize, m.1 as usize];
                let ai_prob = output[index];

                if ai_prob > 0.0 {
                    weights.push((ai_prob * 1000.0) as i32);
                    actions.push((p.x, p.y, m.0, m.1));
                }
            }
        }

        let dist = WeightedIndex::new(weights).unwrap();
        let mut rng = rand::thread_rng();
        let index = dist.sample(&mut rng);
        let action = actions[index];

        Some(gym_env::Action {
            piece: game::find_piece_at_position((action.0, action.1), &state.pieces).unwrap(),
            square: game::Square {
                x: action.2,
                y: action.3,
            },
        })
    }
}
