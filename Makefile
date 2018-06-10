.PHONY: frontend backend

frontend:
	cargo web start \
		--target wasm32-unknown-unknown \
		--auto-reload \
		--host 0.0.0.0 \
		--release \
		--features=frontend

backend:
	RUST_LOG=webapp=trace \
	cargo run \
		--bin backend \
		--release \
		--features=backend
