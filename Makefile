GENERAL_ARGS = --release
FRONTEND_TARGET = $(GENERAL_ARGS) --target wasm32-unknown-unknown
FRONTENT_ARGS = $(FRONTEND_TARGET) --no-default-features --features=frontend
BACKEND_TARGET = $(GENERAL_ARGS)
BACKEND_ARGS = $(BACKEND_TARGET)

.PHONY: frontend backend

ifndef VERBOSE
.SILENT:
else
GENERAL_ARGS += -v
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
