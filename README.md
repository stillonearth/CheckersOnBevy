# Checkers Ugolki game with Bevy and Rust and AI Agent based on Monte Carlo Tree Search (MCTS) with neural evaluation of actions on value functions (AlphaZero) with PyTorch and Python

**Sergei Surovtsev** <<ssurovsev@gmail.com>>
<br />
February 2022

![Шашка](https://i.ibb.co/WDxxKJD/Screenshot-2022-01-13-140430.png)

## Project Description

This project is ground-up introduction to modern game programming with Rust on [Bevy](https://bevyengine.org/) Engine and AI programming with PyTorch. 

In the first part of this project we will implement a Checkers Ugolki game with Bevy game engine on Rust programming language. Then we will implement an OpenAI Gym-compatible environment to train an AI agent to play this game with PyTorch and Python. In last part of this project we will deploy AI agent to Rust Environment.

### Project Goals

* Introduction to Rust programming language
* Introduction to game programming with Bevy engine
* Implementing an OpenAI Gym environment from ground-up
* Training an AI agent to play Checkers Ugolki Game with PyTorch
* Deploying trained model to Rust environment

### Technical Formulation of Problem 

* Set up Rust development environment
* Set up python development environment with PyTorch 1.10+ (CUDA support is desirable)

## 1. Making Checkers Ugolki game with Bevy and Rust

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

## 1.1 checkers-core

```checkers-core``` contains game logic and bevy frontend. It does not contain any network functionality and can be compiled as Desktop (Windows, Linux), Mobile (Android, iOS) or Web Assembly target.

## 1.1.1 Architecture

### Game Rules

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

### Front-End 

```bevy_frontend.rs``` implements Bevy application. 

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

### OpenAI Gym Interface

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

### Exporting modules as a library

In order to use these modules in other projects they have to be exported in `lib.rs`:

```rust
pub mod bevy_frontend;
pub mod game;
pub mod gym_env;

```

## 2. Making OpenAI Gym Environment with Python and Rust

### Architecture

## 3. Training AlphaZero to play Checkers Ugolki with PyTorch and Python

### Mathematical Models

## 4. Deploying to Production with TorchScript and Rust

## Results

## References