# Compiler configuration
GENERAL_ARGS = --release
FRONTEND_TARGET = $(GENERAL_ARGS) --target wasm32-unknown-unknown
FRONTENT_ARGS = $(FRONTEND_TARGET) --no-default-features --features=frontend
BACKEND_TARGET = $(GENERAL_ARGS)
BACKEND_ARGS = $(BACKEND_TARGET)

# API configuration
API_PORT = 30000
API_URL = saschagrunert.de
WS_PATH = /ws
WS_URL = "wss://$(API_URL):$(API_PORT)$(WS_PATH)"
SERVER_URL = "$(API_URL):$(API_PORT)"

.PHONY: frontend frontend_deploy backend

ifndef VERBOSE
.SILENT:
else
GENERAL_ARGS += -v
endif

frontend:
	WS_URL=$(WS_URL) \
	cargo web start \
		$(FRONTENT_ARGS) \
		--auto-reload \
		--host 0.0.0.0

frontend_deploy:
	WS_URL=$(WS_URL) \
	cargo web deploy \
		$(FRONTENT_ARGS)

backend:
	WS_PATH=$(WS_PATH) \
	SERVER_URL=$(SERVER_URL) \
    RUST_LOG=actix_web=info,webapp=trace \
	cargo run \
		$(BACKEND_ARGS) \
		--bin backend
