```bash
cargo install tauri-cli --version "^2.0.0" --locked
cargo tauri dev
RUST_LOG=debug,trace cargo test -p simcraft --lib
cargo clippy --all-targets
```
