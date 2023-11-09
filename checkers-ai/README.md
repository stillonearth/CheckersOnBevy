## 🗀 checkers-ai

### 🐍 Python

`env.py` describes `Env` — a gRPC client that communicates with Rust gRPC server. This `Env` also implements Gym Environment interface.

### 🦀 Rust

`env.rs` uses `tch-rs` to load python-trained model and make call to it. Some caveats:

- Torch Script doesn't play well with `tch-rs` and tensor dimensions get messed up when ts model is loaded to Rust. Because of them model has to be translated from Python to Rust. It's not a big deal for small models, but as model grows can be an issue.
- `checkers-ai` tends to recompile each `cargo build`. This turns sub-1sec compilations to 5-10sec compilations. This is due code-generation on `tch-rs` side.
- Documentation of `tch-rs` is scarce and debugging is non-obvious.
- `tch-rs` utilizes `libcuda` which makes wasm and mobile deployments a problem. For Android there is _PyTorch Mobile_, but then calls from rust have to be wrapped to android calls.

---

## 👾 Training AlphaZero to play 🏁 Checkers with PyTorch and 🐍 Python

Checkers is fully observable turn-based zero-sum game, which means:

- **Fully Observable Game:** Game state is fully known to both players at any moment of the game
- Players make turns in succession
- **Zero-Sum Game:** When one player wins, other looses

Monte-Carlo Search Tree is known method for solving such games. For games with higher dimensionality there is AlphaZero flavor which uses neural network(s) to improve computation efficiency and performance of Search Trees.

### 🧮 Mathematical Models

#### 🌴 Monte-Carlo Tree Search

MCTS algorithm operates on game trees and doesn't need to know game rules. Via self-play it can discover strategies which with sufficient amount of compute outperform human players. [3]

We organize sequence of moves into tree structure. I.e. for each game state node in tree there are n branches where n is number of allowed actions from that state.

A node N on tree T is characterized by:

| var | description            |
| --- | ---------------------- |
| S   | state                  |
| A   | action                 |
| R   | reward                 |
| D   | terminal               |
| Q   | value for both players |
| N   | num visits             |

**Algorithm**

1. Traverse tree from given node `N` to leaf node `L` using _traverse policy_ — non-stochastic policy
2. From a leaf node `L` rollout `n` trajectories until terminal node is reached or computation budged is exhausted using _rollout policy_ — stochastic policy
3. Back-propagate terminal node result through from `L` to root node `R` — visitation counts and values

_Traverse policy_: among node's children chose one with highest Upper Confidence Bound value `Q/(N.N+1) + sqrt(c*log(N.parent.N+1)/(N.N+1))` where `c=sqrt(2)` [2]

UCT weights used to choose nodes during traverse:

```python
def uct(self, node, c=MCTS_C):
    if node.current_player() == 1:
        Q_v = node.q_black
    else:
        Q_v = node.q_white
    N_v = node.number_of_visits + 1
    N_v_parent = node.parent.number_of_visits + 1
```

_Rollout policy_: pick unvisited children node randomly.

#### 0️⃣ AlphaZero

Alpha-zero improves on vanilla MCST by introducing two-headed neural network to evaluate node's value (predict who's winning) and suggest actions to maximize node's value function. This project uses previous work [2] implementation of AlphaZero `checkers-ai/python/monte_carlo_tree.py`

**Algorithm**### 2.3.1 🐍 Python

`env.py` describes `Env` — a gRPC client that communicates with Rust gRPC server. This `Env` also implements Gym Environment interface.

### 🦀 Rust

`env.rs` uses `tch-rs` to load python-trained model and make call to it. Some caveats:

- Torch Script doesn't play well with `tch-rs` and tensor dimensions get messed up when ts model is loaded to Rust. Because of them model has to be translated from Python to Rust. It's not a big deal for small models, but as model grows can be an issue.
- `checkers-ai` tends to recompile each `cargo build`. This turns sub-1sec compilations to 5-10sec compilations. This is due code-generation on `tch-rs` side.
- Documentation of `tch-rs` is scarce and debugging is non-obvious.
- `tch-rs` utilizes `libcuda` which makes wasm and mobile deployments a problem. For Android there is _PyTorch Mobile_, but then calls from rust have to be wrapped to android calls.

---

## 👾 Training AlphaZero to play 🏁 Checkers with PyTorch and 🐍 Python

Checkers is fully observable turn-based zero-sum game, which means:

- **Fully Observable Game:** Game state is fully known to both players at any moment of the game
- Players make turns in succession
- **Zero-Sum Game:** When one player wins, other looses

Monte-Carlo Search Tree is known method for solving such games. For games with higher dimensionality there is AlphaZero flavor which uses neural network(s) to improve computation efficiency and performance of Search Trees.

### 🧮 Mathematical Models

#### 🌴 Monte-Carlo Tree Search

MCTS algorithm operates on game trees and doesn't need to know game rules. Via self-play it can discover strategies which with sufficient amount of compute outperform human players. [3]

We organize sequence of moves into tree structure. I.e. for each game state node in tree there are n branches where n is number of allowed actions from that state.

A node N on tree T is characterized by:

| var | description            |
| --- | ---------------------- |
| S   | state                  |
| A   | action                 |
| R   | reward                 |
| D   | terminal               |
| Q   | value for both players |
| N   | num visits             |

**Algorithm**

1. Traverse tree from given node `N` to leaf node `L` using _traverse policy_ — non-stochastic policy
2. From a leaf node `L` rollout `n` trajectories until terminal node is reached or computation budged is exhausted using _rollout policy_ — stochastic policy
3. Back-propagate terminal node result through from `L` to root node `R` — visitation counts and values

_Traverse policy_: among node's children chose one with highest Upper Confidence Bound value `Q/(N.N+1) + sqrt(c*log(N.parent.N+1)/(N.N+1)
def rollout_policy(self, node):

    state = node.prepare_state()
    state_tensor = torch.from_numpy(state).float().to(self.device).unsqueeze(0)
    probs_tensor, _ = self.actor_critic_network(state_tensor)
    probs_tensor = self.correct_probs_with_possible_actions(node, probs_tensor) \
        .cpu().detach().numpy()
    actions = np.argwhere(probs_tensor>0).tolist()
    probs = np.array([probs_tensor[m[0], m[1], m[2], m[3]] for m in actions])
    probs /= np.sum(probs)
    if len(probs) == 0:
        return None, 1.0
    index = np.random.choice(np.arange(len(probs)), p=probs)

    return actions[index], probs[index]
```

### 📉 Loss Function

Loss function to train AlphaZero used in this project is:

![formula](<https://render.githubusercontent.com/render/math?math=l=-\pi^Tlog(p)%2b(v-z)^2>)

### 🧠 Neural Networks

Neural network used in this project is attributed to [5]. This is two-headed network used to predict value and policy functions.

```python
import torch
import torch.nn as nn
import torch.nn.functional as F

class ActorCritic(nn.Module):

    def __init__(self, board_size=BOARD_SIZE):
        super(ActorCritic, self).__init__()

        self.board_size = board_size
        self.conv1 = nn.Conv2d(1, 64, kernel_size=3, padding=1)
        self.conv2 = nn.Conv2d(64, 128, kernel_size=3, padding=1)
        self.conv3 = nn.Conv2d(128, 128, kernel_size=3, padding=1)
        self.conv4 = nn.Conv2d(128, 8192, kernel_size=3, padding=1)
        self.layer1 = nn.Linear(8192, 4096)

    def forward(self, x):

        x = x.unsqueeze(1)
        x = F.relu(self.conv1(x))
        x = F.relu(self.conv2(x))
        x = F.max_pool2d(x, 2)
        x = F.relu(self.conv3(x))
        x = F.max_pool2d(x, 2)
        x = F.relu(self.conv4(x))
        x = F.max_pool2d(x, 2)
        x = F.dropout(x, p=0.2, training=self.training)
        x = x.view(-1, 8192)

        prob = F.hardsigmoid(self.layer1(x))
        value = F.hardtanh(self.layer2(x))

        return prob.view(-1, 8, 8, 8, 8), value.view(-1, 1)
```

---


## Training an Agent

1. Start a game logic rpc server `cargo run --bin checkers-server`
2. Train a model in jupyter notebook `checkers-ai/python/MTSC-Checkers.ipynb`



## 🍻 Acknowledgements

- [A1] Gym Go Environment, Eddie Huang, August 2019 — May 2021, https://github.com/aigagror/GymGo

## 📜 References

- [1] **Chess game in Rust using Bevy**, _guimcaballero_, Nov 16th 2020, <br /> https://caballerocoll.com/blog/bevy-chess-tutorial/
- [2] **Reimplementing Alpha-Zero for board game of Go**, _Sergei Surovtsev_, December 2019, <br />https://github.com/cwiz/guided_monte_carlo_tree-search/blob/master/Tree-Search.ipynb
- [3] **CS234 Notes - Lecture 14 Model Based RL, Monte-Carlo Tree Search**, _Anchit Gupta, Emma Brunskill_, June 2018, <br />https://web.stanford.edu/class/cs234/CS234Win2019/slides/lnotes14.pdf
- [4] **A general reinforcement learning algorithm that masters chess, shogi and Go through self-play**, _Silver, David and Hubert, Thomas and Schrittwieser, Julian and Antonoglou, Ioannis and Lai, Matthew and Guez, Arthur and Lanctot, Marc and Sifre, Laurent and Kumaran, Dharshan and Graepel, Thore and others_, Science 362 (6419): 1140--1144 (2018), <br />https://kstatic.googleusercontent.com/files/2f51b2a749a284c2e2dfa13911da965f4855092a179469aedd15fbe4efe8f8cbf9c515ef83ac03a6515fa990e6f85fd827dcd477845e806f23a17845072dc7bd
- [5] **Udacity Deep Reinforcement Learning Weekly Webinar**, 2019, <br/>https://www.youtube.com/watch?v=X72vKonfzCk
- [6] **Zero performance**, Gian-Carlo Pascutto, October 2020, <br /> https://web.archive.org/web/20190205013627/http://computer-go.org/pipermail/computer-go/2017-October/010307.html
