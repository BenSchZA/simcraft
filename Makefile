setup:
	cargo install tauri-cli --version "^2.0.0" --locked

dev:
	cargo tauri dev

test:
	RUST_LOG=debug,info cargo test -p simcraft

lint:
	cargo clippy --all-targets
