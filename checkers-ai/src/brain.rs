use rand;
use rand::{distributions::WeightedIndex, prelude::Distribution};
use tch;
use tch::{nn, nn::Module, nn::OptimizerConfig, Device};

use checkers_core::game;
use checkers_core::gym_env;

pub struct Brain {
    model: ActorCritic,
}

#[derive(Debug)]
struct ActorCritic {
    conv1: nn::Conv2D,
    conv2: nn::Conv2D,
    conv3: nn::Conv2D,

    act_conv1: nn::Conv2D,
    act_fc1: nn::Linear,
}

impl ActorCritic {
    fn new(vs: &nn::Path) -> ActorCritic {
        let conv2d_cfg = nn::ConvConfigND::<i64> {
            padding: 1,
            ..Default::default()
        };

        let conv1 = nn::conv2d(vs, 1, 32, 5, conv2d_cfg);
        let conv2 = nn::conv2d(vs, 32, 64, 3, conv2d_cfg);
        let conv3 = nn::conv2d(vs, 64, 128, 3, conv2d_cfg);

        let act_conv1 = nn::conv2d(vs, 128, 4, 1, conv2d_cfg);
        let act_fc1 = nn::linear(vs, 256, 4096, Default::default());
        ActorCritic {
            conv1,
            conv2,
            conv3,
            act_conv1,
            act_fc1,
        }
    }
}

impl nn::Module for ActorCritic {
    fn forward(&self, xs: &tch::Tensor) -> tch::Tensor {
        let x = xs.view([-1, 1, 8, 8]);
        let x = x.apply(&self.conv1).relu();
        let x = x.apply(&self.conv2).relu();
        let x = x.apply(&self.conv3).relu();
        let x = x.apply(&self.act_conv1).relu();

        let x = x
            .view([-1, 256])
            .apply(&self.act_fc1)
            .sigmoid()
            .view([8, 8, 8, 8]);

        return x;
    }
}

impl Brain {
    pub fn new() -> Brain {
        // let mut model =
        //     tch::CModule::load("C:\\Users\\Sergei\\git\\checkers\\checkers-ai\\actor_network.pt")
        //         .unwrap();

        // model.to(tch::Device::Cpu, tch::kind::Kind::Float, true);

        let mut vs = nn::VarStore::new(Device::Cpu);
        let model = ActorCritic::new(&vs.root());
        vs.load("C:\\Users\\Sergei\\git\\checkers\\checkers-ai\\actor_network.pt");

        Brain { model }
    }

    pub fn choose_action(&self, state: game::GameState) -> Option<gym_env::Action> {
        let multiplier = match state.turn.color {
            game::Color::Black => 1,
            game::Color::White => -1,
        };

        let mut input = tch::Tensor::zeros(&[8, 8], tch::kind::FLOAT_CPU);

        for p in state.pieces.iter() {
            let value = (multiplier
                * match p.color {
                    game::Color::Black => 1,
                    game::Color::White => -1,
                }) as f32;

            let index = tch::Tensor::of_slice(&[p.x, p.y]).to_kind(tch::Kind::Int64);
            let mut tensor = input.index_select(1, &index);
            tensor = tch::Tensor::of_slice(&[value]);
        }

        let input = input.unsqueeze(1);

        let result = input.apply(&self.model).squeeze();

        let allowed_moves = state.moveset;

        let mut actions: Vec<(u8, u8, u8, u8)> = Vec::new();
        let mut weights: Vec<i32> = Vec::new();

        for p in &state.pieces {
            for m in allowed_moves[p.id as usize].iter() {
                if p.color != state.turn.color {
                    continue;
                }

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
