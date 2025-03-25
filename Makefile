setup:
	cargo install tauri-cli --version "^2.0.0" --locked

dev:
	cargo tauri dev

test: test_lib test_web

test_lib:
	RUST_LOG=debug,info cargo test -p simcraft

test_web:
	RUST_LOG=debug,info wasm-pack test --node crates/simcraft_web

lint:
	cargo clippy --all-targets
