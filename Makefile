RELEASE := --release
FRONTEND_TARGET := $(RELEASE) --target wasm32-unknown-unknown
FRONTENT_ARGS := $(FRONTEND_TARGET) --features=frontend
BACKEND_TARGET := $(RELEASE)
BACKEND_ARGS := $(BACKEND_TARGET) --features=backend

.PHONY: frontend backend

ifndef VERBOSE
.SILENT:
endif

frontend:
	cargo web start \
		$(FRONTENT_ARGS) \
		--auto-reload \
		--host 0.0.0.0

frontend_deploy:
	cargo web deploy \
		$(FRONTENT_ARGS)

backend:
	RUST_LOG=webapp=trace \
	cargo run \
		$(BACKEND_ARGS) \
		--bin backend
