# Setup

## Rust
- [ ] Install rustup
- [ ] Add `wasm32-unknown-unknown` toolchain 

## First run of wasm
```bash
export PATH="$(pwd)/.cargo/bin:$PATH"  # The flake does this automatically. Could use direnv too to do this
cargo install --git https://github.com/TheBevyFlock/bevy_cli --tag cli-v0.1.0-alpha.2 --locked bevy_cli --root .cargo
cargo install wasm-bindgen-cli --root .cargo
```

## Now run!
```bash
bevy run web
```
