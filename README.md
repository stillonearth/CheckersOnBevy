# Checkers Ugolki game on Rust and Bevy, OpenAI Gym Environment and AI Agent based on Monte Carlo Tree Search (MCTS) with neural heuristics (AlphaZero) on Python and PyTorch

**Sergei Surovtsev** <<ssurovsev@gmail.com>>
<br />
February 2022

![Шашка](https://i.ibb.co/WDxxKJD/Screenshot-2022-01-13-140430.png)

## Project Description

This project is ground-up introduction to modern game programming with Rust on [Bevy](https://bevyengine.org/) Engine and AI programming with PyTorch. 

In the first part of this project we will implement a Checkers Ugolki game with Bevy game engine on Rust programming language. Then we will implement an OpenAI Gym-compatible environment to train an AI agent to play this game with PyTorch and Python. In last part of this project we will deploy AI agent to Rust environment targeting Desktop (Windows, Linux).

### Project Goals

* Introduction to Rust programming language
* Introduction to game programming with Bevy engine
* Implementing an OpenAI Gym environment from ground-up
* Training an AI agent to play Checkers Ugolki Game with PyTorch
* Deploying trained model to Rust environment

### Technical Formulation of Problem 

* Set up Rust development environment
* Set up Python development environment with PyTorch 1.10+ (CUDA support is desirable)

***

## 1. Making Checkers Ugolki game with Bevy and Rust

On high-level project is structured with [workspace](https://doc.rust-lang.org/cargo/reference/manifest.html#the-workspace-field) feature of Cargo.toml.

Project is organized in following manner:

```
CheckersOnBevy
 |--checkers-core   # Contains bevy application and game core mechanics. Can run standalone game.
 |--checkers-app    # Bevy front-end application
 |   |--assets      # Models, Fonts and pictures
 |--checkers-ai     # Python code to train a model and Rust deployment
 |--checkers-server # gRPC server with game core mechanics
 `--checkers-client # Bevy frontend that connects with server.
```

The reason is that we want our game logic be decoupled from a game front-end (Bevy application) and be accessible to other languages with gRPC API. Application can be run with Client-Server model or as a standalone one. ```checkers-server``` is also used in 2nd part of this project where we train a Neural Network to play the game.

## 1.1 🗀 checkers-core

```checkers-core``` contains game logic and bevy frontend. It does not contain any network functionality and can be compiled as Desktop (Windows, Linux), Mobile (Android, iOS) or Web Assembly target.

### 1.1.1 Game Rules

`game.rs` describes game rules. It contains following entities:

* ```enum Color{White, Black}```
* ```enum MoveType {Invalid,JumpOver,Regular,Pass}```
* ```enum GameTermination {White,Black,Draw,WhiteMoveLimit,BlackMoveLimit,Unterminated}```
* ```struct PlayerTurn{color:Color, turn_count, chain_count, chain_piece_id}```
* ```struct Piece{x, y, id}```
* ```struct Square{x, y}```
* ```type Position = (u8, u8)``` 
* ```struct GameState{pieces:Vec<Piece>, turn:PlayerTurn, moveset:[Vec<Position>;18]}```
* ```struct Game{state: GameState, squares: Vec<Square>}```

This is pure Rust module, but ```Piece``` and ```Square``` are decorated with bevy's ```Component``` decorator which is needed for Entity-Component-System (ECS) pattern used in Bevy.

### 1.1.2 OpenAI Gym Interface

[OpenAI Gym](https://gym.openai.com/) describes Environment interface in following way:

```python
env = Environment()
observation = env.reset(state)
for _ in range(1000):
  env.render()
  action = env.action_space.sample() # your agent here (this takes random actions)
  observation, reward, done, info = env.step(action)

  if done:
    observation = env.reset()
env.close()
```

```gym_env.rs``` implements environment in Rust and introduces a couple of new entities:

* ```struct Action {piece: Piece,square: game::Square}```
* ```struct Step {obs: GameState,action, reward, is_done}```
* ```struct CheckersEnv {game: Game, initial_state: GameState}```

```CheckersEnv``` has following methods:

* ```fn reset(state: Option<game::GameState>) -> GameState```
* ```fn step(action: Action) -> Step```

In part 2 of this project ```CheckersEnv``` is exposed as gRPC server and python client is implemented to communicate with it.

### 1.1.4 Exporting modules as a library

In order to use these modules in other projects they have to be exported via `lib.rs`:

***

## 1.2 🗀 checkers-app

### 1.2.1 Front-End 

```bevy_frontend.rs``` implements Bevy application which is influenced by [1].

Bevy uses ECS pattern to describe game logic. It suggests to organize logic in following manner:

* *Entities* — game engine objects such as meshes, groups of meshes 
* *Components* — attributes that can be matched to entities
* *Systems* — functions that operate on entities and their components 

In Bevy there is also notion on *Resources* which are similar to global variables or singletons. *Events* are used to pass messages between systems. *Plugins* organize Systems and Resources.

**Plugins and Systems**

*BoardPlugin* describes game board and high-level events such as selection of square, movement and game termination.

```
 BoardPlugin
  |
  |--[Resources ]
  |    |--SelectedSquare
  |    `--SelectedPiece
  |
  |--[Startup Systems]
  |    `--create_board
  |
  |--[Events]
  |   `--event_square_selected
  |
  |--[Components]
  |    |--Square
  |    `--Piece
  |
  |--[Systems]
  |    |--player_move
  |    |     `--update_entity_pieces
  |    |
  |    |
  |    |--computer_move
  |    `--check_game_termination
  |
  `--[Plugins]
       `--PiecesPlugin
```

*PiecesPlugin* describes pieces, movement animation and highlighting.

```
 PiecesPlugin
  |
  |--[Startup Systems]
  |    `--create_pieces
  |
  |--[Events]
  |   `--event_piece_moved
  |
  |--[Systems]
  |    `--highlight_piece
  |
  `--[Plugins]
       `--TweeningPlugin
```

*UIPlugin* describes buttons and game state text label.

```
 UIPlugin
  |
  |--[Startup Systems]
  |    `--init_text
  |
  |--[Events]
  |   `--event_piece_moved
  |
  `--[Systems]
       |--next_move_text_update
       `--button_system
```

## 2. Making OpenAI Gym Environment with Python and Rust

## 2.1 🗀 checkers-server

We want our Python Gym environment to run simulation, correct it state and take screenshots. We could have wrapped entire application to [pyo3](https://github.com/PyO3/pyo3) but that would require running ```bevy_frontend``` on non-main thread, while communicating with ```gym_env``` which would share game state with bevy app. Unfortunately there are number of limitations limiting us from this path:

1. Bevy frontend needs main thread for it's event loop
2. Sharing game state with bevy app and environment is not straightforward due to Rust limitation on multiple mutable references to single variable

Because of that ```checkers-server``` implements a gRPC server wrapping ```CheckersEnv```. It is [Tonic](https://github.com/hyperium/tonic) gRPC server which assumes that environment can be accessed simultaneously from multiple threads so ```CheckersEnv``` is wrapped in mutex in atomic reference counter ```Arc<Mutex<CheckersEnv>>```. 

```proto/environment.proto``` describes service contract with [Protocol Buffers](https://developers.google.com/protocol-buffers/). We don't fully describe service fields because all responses are serialized to json entities from *§1.1.1 Game Rules* and *§1.1.3  OpenAI Gym Interface*. Note that production services should describe all fields in Protocol Buffers contracts.


## 2.2 🗀 checkers-client

A client that communicates with ```checkers-server```. You can run multiple instances — a multiplayer game. We also use it to visualize shared game state in OpenAI Gym Environment

## 2.3 🗀 checkers-ai

### 2.3.1 Python

```env.py``` describes ```Env``` — a gRPC client that communicates with Rust gRPC server. This ```Env``` also implements OpenAI Gym Environment interface.

### 2.3.2 Rust

```env.rs``` uses ```tch-rs``` to load python-trained model and make call to it. Some caveats: 

* Torch Script doesn't play well with ```tch-rs``` and tensor dimensions get messed up when ts model is loaded to Rust. Because of them model has to be translated from Python to Rust. It's not a big deal for small models, but as model grows can be an issue.
* ```checkers-ai``` tends to recompile each ```cargo build```. This turns sub-1sec compilations to 5-10sec compilations. This is due code-generation on ```tch-rs``` side.
* Documentation of ```tch-rs``` is scarce and debugging is non-obvious.
* ```tch-rs``` utilizes ```libcuda``` which makes wasm and mobile deployments a problem. For Android there is *PyTorch Mobile*, but then calls from rust have to be wrapped to android calls. 

***

## 3. Training AlphaZero to play Checkers Ugolki with PyTorch and Python

Checkers is fully observable turn-based zero-sum game, which means:

* **Fully Observable Game:** Game state is fully known to both players at any moment of the game
* Players make turns in succession
* **Zero-Sum Game:** When one player wins, other looses

Monte-Carlo Search Tree is known method for solving such games. For games with higher dimensionality there is AlphaZero flavor which uses neural network(s) to improve computation efficiency and performance of Search Trees.

### 3.1 Mathematical Models

#### 3.1.1 Monte-Carlo Tree Search

MCTS algorithm operates on game trees and doesn't need to know game rules. Via self-play it can discover strategies which with sufficient amount of compute outperform human players. [3]

We organize sequence of moves into tree structure. I.e. for each game state node in tree there are n branches where n is number of allowed actions from that state.

A node N on tree T is characterized by:

|  |   |
|--|---|
|S |state|
|A |action|
|R |reward|
|D |terminal|
|Q |value for both players |
|N |num visits|


**Algorithm**

1. Traverse tree from given node `N` to leaf node `L` using *traverse policy* — non-stochastic policy
2. From a leaf node `L` rollout `n` trajectories until terminal node is reached or computation budged is exhausted using *rollout policy* — stochastic policy
3. Back-propagate terminal node result through from `L` to root node `R` — visitation counts and values

*Traverse policy*: among node's children chose one with highest Upper Confidence Bound value ```Q/(N.N+1) + sqrt(c*log(N.parent.N+1)/(N.N+1))``` where ```c=sqrt(2)``` [2]

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

*Rollout policy*: pick unvisited children node randomly.

#### 3.1.2 AlphaZero

Alpha-zero improves on vanilla MCST by introducing two-headed neural network to evaluate node's value (predict who's winning) and suggest actions to maximize node's value function. This project uses previous work [2] implementation of AlphaZero ```checkers-ai/python/monte_carlo_tree.py```

**Algorithm**

AlphaZero uses slightly modified metric to choose nodes during traverse:

```python
def uct(self, node, c=MCTS_C):
    N_v = node.number_of_visits + 1
    N_v_parent = node.parent.number_of_visits + 1
    V_current = self.estimate_node_value(node)

    return np.sum(V_current * node.prob + c*np.sqrt(np.log(N_v_parent)/N_v))
```

During rollout we use neural network to choose actions with higher probabilities to maximize value function:

```python
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

### 3.1.2.1 Loss Function

Loss function to train AlphaZero used in this project is:

![formula](https://render.githubusercontent.com/render/math?math=l=-\pi^Tlog(p)%2b(v-z)^2)

### 3.2 Neural Networks

Neural network used in this project is attributed to [5]. This is two-headed network used to predict value and policy functions.

```python
import torch
import torch.nn as nn
import torch.nn.functional as F

class ActorCritic(nn.Module):

    def __init__(self, board_size=8):
        super(ActorCritic, self).__init__()
        
        self.board_size = board_size
        self.conv1 = nn.Conv2d(1, 64, kernel_size=3, padding=1)
        self.conv2 = nn.Conv2d(64, 128, kernel_size=3, padding=1)
        self.conv3 = nn.Conv2d(128, 128, kernel_size=3, padding=1)
        self.conv4 = nn.Conv2d(128, 128, kernel_size=3, padding=1)
        self.layer1 = nn.Linear(128, 64)
        self.layer2 = nn.Linear(128, 1)
        
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
        x = x.view(-1, 128)
        
        prob = F.hardsigmoid(self.layer1(x))
        value = F.hardtanh(self.layer2(x))

        return prob.view(-1, self.board_size, self.board_size), value.view(-1, 1)
```

***

## 4. Production

## 4.1 Deploying to Desktop (Windows)

```cargo build``` to build all binaries.

***

## Results 

### Training Results

Google-produced Alpha-Zero trained on Google TPUs which faster than consumer-available hardware. To achieve human and above-human performance significant compute time is needed on consumer hardware. In this project network is trained to validate performance on Go Environment [2] for 1000 iterations and then same network is trained for game of Checkers.

According to [6] over 1700 years are required to reproduce Google AlphaZero weights on consumer hardware, which makes reproducibility a problem.

#### Go

Previous work [2] with some fixes with  network architecture changes [5]: https://github.com/stillonearth/CheckersOnBevy/blob/master/checkers-ai/python/MTSC-Go.ipynb

#### Checkers Ugolki

Python Jupyter Notebook with training details: https://github.com/stillonearth/CheckersOnBevy/blob/master/checkers-ai/python/MTSC-Ugolki.ipynb

### Rust, Bevy and Torch 

Subjective state of these instruments (February 2022):

* **Rust & Bevy:**  Bevy is suitable for implementing novel training environments because of stability, memory safety and broad ecosystem of plugins. Bevy doesn't cost you anything, distributed with double MIT, Apache 2.0 and can be freely used in research and production. It can also target Web, Desktop and Mobile making it prime choice for rapid prototyping.
* **PyTorch:** Has been stable enough that code from 2019 has been used with minor modifications. Deploying to mobile is still a struggle. wasm isn't a option for model deployment too, but there is option of ONNX.js for javascript model deployment.

***

## Acknowledgements

* [A1] OpenAI Gym Go Environment, Eddie Huang, August 2019 — May 2021, https://github.com/aigagror/GymGo

## References

* [1] **Chess game in Rust using Bevy**, *guimcaballero*, Nov 16th 2020, <br /> https://caballerocoll.com/blog/bevy-chess-tutorial/
* [2] **Reimplementing Alpha-Zero for board game of Go**, *Sergei Surovtsev*, December 2019, <br />https://github.com/cwiz/guided_monte_carlo_tree-search/blob/master/Tree-Search.ipynb
* [3] **CS234 Notes - Lecture 14 Model Based RL, Monte-Carlo Tree Search**, *Anchit Gupta, Emma Brunskill*, June 2018, <br />https://web.stanford.edu/class/cs234/CS234Win2019/slides/lnotes14.pdf
* [4] **A general reinforcement learning algorithm that masters chess, shogi and Go through self-play**, *Silver, David and Hubert, Thomas and Schrittwieser, Julian and Antonoglou, Ioannis and Lai, Matthew and Guez, Arthur and Lanctot, Marc and Sifre, Laurent and Kumaran, Dharshan and Graepel, Thore and others*, Science 362 (6419): 1140--1144 (2018), <br />https://kstatic.googleusercontent.com/files/2f51b2a749a284c2e2dfa13911da965f4855092a179469aedd15fbe4efe8f8cbf9c515ef83ac03a6515fa990e6f85fd827dcd477845e806f23a17845072dc7bd
* [5] **Udacity Deep Reinforcement Learning Weekly Webinar**, 2019, <br/>https://www.youtube.com/watch?v=X72vKonfzCk
* [6] **Zero performance**, Gian-Carlo Pascutto, October 2020, <br /> https://web.archive.org/web/20190205013627/http://computer-go.org/pipermail/computer-go/2017-October/010307.html