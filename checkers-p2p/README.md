# Checkers online

[build bevy app for wasm](https://bevy-cheatbook.github.io/platforms/wasm.html)

```bash
export CARGO_TARGET_WASM32_UNKNOWN_UNKNOWN_RUNNER=wasm-server-runner
cargo run --target wasm32-unknown-unknown --bin checkers-p2p
```