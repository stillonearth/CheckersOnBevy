# CheckersOnBevy

https://user-images.githubusercontent.com/97428129/202233088-5ad38413-e035-4750-8273-e5475080347d.mp4

A checkers app with:

- AI: 🧠 Train agents and play against NN-trained opponent
- P2P: Play over Veilid 

## 📋 Changelog

- **0.3.0** `bevy` updated to 0.8.1
- **0.4.0** `tch-rs` switched to `tract-onnx `
- **0.4.1** `bevy` updated to 0.9.0
- **0.5.0** `bevy` updated to 0.11
- **0.6.0** `bevy` checkers-p2p to play over network


---

## 🏗️ Building 🏁 Checkers Game with 🕊 Bevy and 🦀 Rust

On high-level project is structured with [workspace](https://doc.rust-lang.org/cargo/reference/manifest.html#the-workspace-field) feature of Cargo.toml.

Project is organized in following manner:

```
CheckersOnBevy
 |--checkers-core   # Contains bevy application and game core mechanics. Can run standalone game.
 |--checkers-app    # Bevy front-end application
 |   |--assets      # Models, Fonts and pictures
 |--checkers-ai     # Python code to train a model and Rust deployment
 |--checkers-p2p    # Play over p2p network
 |--checkers-server # gRPC server with game core mechanics
 `--checkers-client # Bevy frontend that connects with server.
```

### 📝 Usage

1. Install pytorch and rust
2. git clone repository
3. Build project with cargo build
4. cargo run --bin checkers-app


## Usage

```
cargo run --bin checkers-p2p # run p2p app
cargo run --bin checkers-app # run vsai app
cargo run --bin checkers-server # run server to train ai
cargo run --bin checkers-client # run client to see AI training process (see checkers-ai)
```
