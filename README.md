# Checkers Ugolki game on Rust and Bevy, OpenAI Gym Environment and AI Agent based on Monte Carlo Tree Search (MCTS) with neural heuristics (AlphaZero) on Python and PyTorch

**Sergei Surovtsev** <<ssurovsev@gmail.com>>
<br />
February 2022

![Шашка](https://i.ibb.co/WDxxKJD/Screenshot-2022-01-13-140430.png)

## Project Description

This project is ground-up introduction to modern game programming with Rust on [Bevy](https://bevyengine.org/) Engine and AI programming with PyTorch. 

In the first part of this project we will implement a Checkers Ugolki game with Bevy game engine on Rust programming language. Then we will implement an OpenAI Gym-compatible environment to train an AI agent to play this game with PyTorch and Python. In last part of this project we will deploy AI agent to Rust environment targeting Desktop (Windows, Linux) and Web (wasm).

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

![Bevy](https://gamefromscratch.com/wp-content/uploads/2020/08/BevyRustGameEngine.png)

On high-level project is structured with [workspace](https://doc.rust-lang.org/cargo/reference/manifest.html#the-workspace-field) feature of Cargo.toml.

Project is organized in following manner:

```
CheckersOnBevy
 |--checkers-core # Contains bevy application and game core mechanics. Can run standalone game.
 |   |--src
 |   |   |--main.rs
 |   |   `--lib.rs
 |   |--assets
 |   `--Cargo.toml
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

### 1.1.2 Front-End 

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
  |    |--click_square
  |    |     `--update_entity_pieces
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

*PiecesPlugin* describes buttons and game state text label.

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

### 1.1.3 OpenAI Gym Interface

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

## 2. Making OpenAI Gym Environment with Python and Rust

![Gym](https://miro.medium.com/max/659/1*Y2mmrAOmmb1pNCVGINJxQA.png)

## 2.1 🗀 checkers-server

We want our Python Gym environment to run simulation, correct it state and take screenshots. We could have wrapped entire application to [pyo3](https://github.com/PyO3/pyo3) but that would require running ```bevy_frontend``` on non-main thread, while communicating with ```gym_env``` which would share game state with bevy app. Unfortunately there are number of limitations limiting us from this path:

1. Bevy frontend needs main thread for it's event loop
2. Sharing game state with bevy app and environment is not straightforward due to Rust limitation on multiple mutable references to single variable

Because of that ```checkers-server``` implements a gRPC server wrapping ```CheckersEnv```. It is [Tonic](https://github.com/hyperium/tonic) gRPC server which assumes that environment can be accessed simultaneously from multiple threads so ```CheckersEnv``` is wrapped in mutex in atomic reference counter ```Arc<Mutex<CheckersEnv>>```. 

```proto/environment.proto``` describes service contract with [Protocol Buffers](https://developers.google.com/protocol-buffers/). We don't fully describe service fields because all responses are serialized to json entities from *§1.1.1 Game Rules* and *§1.1.3  OpenAI Gym Interface*. Note that production services should describe all fields in Protocol Buffers contracts.


## 2.2 🗀 checkers-client

A client that communicates with ```checkers-server```. You can run multiple instances — a multiplayer game. We also use it to visualize shared game state in OpenAI Gym Environment

## 2.3 🗀 checkers-ai

```env.py``` describes ```Env``` — a gRPC client that communicates with Rust gRPC server. This ```Env``` also implements OpenAI Gym Environment interface.

***

## 3. Training AlphaZero to play Checkers Ugolki with PyTorch and Python

![PyTorch](https://cdn.windowsreport.com/wp-content/uploads/2020/07/Microsoft-is-taking-over-PyTorch-for-Windows-from-Facebook.jpg)

Checkers is fully observable turn-based zero-sum game, which means:

* **Fully Observable Game:** Game state is fully known to both players at any moment of the game
* Players make turns in succession
* **Zero-Sum Game:** When one player wins, other looses

Monte-Carlo Search Tree is known method for solving such games. For games with higher dimensionality there is AlphaZero flavor which uses neural network(s) to improve computation efficiency and performance of Search Trees.

### 3.1 Mathematical Models

#### 3.1.1 Monte-Carlo Search Trees

![MCST](https://images.novatech.co.uk/2020/blog/monte-carlo-search-tree-algorithm.png)

MCST algorithm operates on game trees and doesn't need to know game rules. Via self-play it can discover strategies which with sufficient amount of compute outperform human players. [3]

We organize sequence of moves into tree structure. I.e. for each game state node in tree there are n branches where n is number of allowed actions from that state.

A node N on tree T is characterized by:

* S: State
* A: Action
* R: Reward
* D: Whether node is terminal, game end reached
* q_black: value of node for blacks winning starting from this node
* q_white: value of node for whites winning starting from this node
* N: Number of visits

**Vanilla Monte-Carlo Play Tree Algorithm**

1. Traverse tree from given node `N` to leaf node `L` using `traverse policy` — non-stochastic
2. From a leaf node `L` rollout `n` trajectories until terminal node is reached or computation budged is exhausted
3. Back-propagate terminal node result through from `L` to root node `R`

Traverse Policy: among node's children chose one with highest Upper Confidence Bound value ```Q/(N.N+1) + sqrt(c*log(N.parent.N+1)/(N.N+1))``` where ```c=sqrt(2)```. See [2] for details.

#### 3.1.2 AlphaZero

Alpha-zero improves on vanilla MCST by introducing two-headed neural network to evaluate node's value (predict who's winning) and suggest actions to maximize node's value function. See [2] for details and ```checkers-ai/monte_carlo_tree.py``` for implementation.

### 3.2 Neural Networks

***

## 4. Deploying to Production with TorchScript and Rust [DRAFT]

![Deploy](https://res.infoq.com/presentations/pytorch-torchscript-botorch/en/slides/sl43-1566323726996.jpg)

## 4.1 🗀 checkers-ai

***

## Results [DRAFT]

### Training Results

### Rust, Bevy and Torch 

Subjective state of these instruments (February 2022):

* **Rust & Bevy:**  Bevy is suitable for implementing novel training environments because of stability, memory safety and broad ecosystem of plugins. Bevy doesn't cost you anything, distributed with double MIT, Apache 2.0 and can be freely used in research and production. It can also target Web, Desktop and Mobile making it prime choice for rapid prototyping.
* **PyTorch:** Has been stable enough that code from 2019 has been used with minor modifications.

***

## References

* [1] Chess game in Rust using Bevy, guimcaballero, Nov 16th 2020, https://caballerocoll.com/blog/bevy-chess-tutorial/
* [2] Reimplementing Alpha-Zero for board game of Go, Sergei Surovtsev, December 2019, https://github.com/cwiz/guided_monte_carlo_tree-search/blob/master/Tree-Search.ipynb
* [3] CS234 Notes - Lecture 14 Model Based RL, Monte-Carlo Tree Search, Anchit Gupta, Emma Brunskill, June 2018, https://web.stanford.edu/class/cs234/CS234Win2019/slides/lnotes14.pdf