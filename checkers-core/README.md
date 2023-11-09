## checkers-core

`checkers-core` contains game logic and bevy frontend. It does not contain any network functionality and can be compiled as Desktop (Windows, Linux), Mobile (Android, iOS) or Web Assembly target.

### 🎲 Game Rules

`game.rs` describes game rules. It contains following entities:

- `enum Color{White, Black}`
- `enum MoveType {Invalid,JumpOver,Regular,Pass}`
- `enum GameTermination {White,Black,Draw,WhiteMoveLimit,BlackMoveLimit,Unterminated}`
- `struct PlayerTurn{color:Color, turn_count, chain_count, chain_piece_id}`
- `struct Piece{x, y, id}`
- `struct Square{x, y}`
- `type Position = (u8, u8)`
- `struct GameState{pieces:Vec<Piece>, turn:PlayerTurn, moveset:[Vec<Position>;18]}`
- `struct Game{state: GameState, squares: Vec<Square>}`

This is pure Rust module, but `Piece` and `Square` are decorated with bevy's `Component` decorator which is needed for Entity-Component-System (ECS) pattern used in Bevy.

### 🏋🏿 Gym Interface

[Gym](https://gym.com/) describes Environment interface in following way:

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

`gym_env.rs` implements environment in Rust and introduces a couple of new entities:

- `struct Action {piece: Piece,square: game::Square}`
- `struct Step {obs: GameState,action, reward, is_done}`
- `struct CheckersEnv {game: Game, initial_state: GameState}`

`CheckersEnv` has following methods:

- `fn reset(state: Option<game::GameState>) -> GameState`
- `fn step(action: Action) -> Step`

In part 2 of this project `CheckersEnv` is exposed as gRPC server and python client is implemented to communicate with it.

### 📚 Exporting modules as a library

In order to use these modules in other projects they have to be exported via `lib.rs`:

---
