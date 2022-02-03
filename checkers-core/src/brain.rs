use rand;
use rand::{distributions::WeightedIndex, prelude::Distribution};
use tch;

use crate::game;
use crate::gym_env;

pub struct Brain {
    model: tch::CModule,
}

impl Brain {
    pub fn new() -> Brain {
        let mut model =
            tch::CModule::load("C:\\Users\\Sergei\\git\\checkers\\checkers-ai\\actor_network.pt")
                .unwrap();

        model.to(tch::Device::Cpu, tch::kind::Kind::Float, true);

        Brain { model }
    }

    pub fn choose_action(&self, state: game::GameState) -> Option<gym_env::Action> {
        let input = tch::Tensor::zeros(&[8; 8], tch::kind::FLOAT_CPU).unsqueeze(0);

        println!("{}", input.size()[2]);

        let result = input.apply(&self.model).squeeze();

        let allowed_moves = state.moveset;

        let mut actions: Vec<(u8, u8, u8, u8)> = Vec::new();
        let mut weights: Vec<i32> = Vec::new();

        for p in &state.pieces {
            for m in allowed_moves[p.id as usize].iter() {
                let ai_prob =
                    result.double_value(&[p.x as i64, p.y as i64, m.0 as i64, m.1 as i64]);

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
