# Compiler configuration
GENERAL_ARGS = --release
FRONTEND_TARGET = $(GENERAL_ARGS) --target wasm32-unknown-unknown
FRONTENT_ARGS = $(FRONTEND_TARGET) --no-default-features --features=frontend
BACKEND_TARGET = $(GENERAL_ARGS)
BACKEND_ARGS = $(BACKEND_TARGET)

# API configuration
API_PORT = 443
API_URL = 0.0.0.0
WS_PATH = /ws
WS_URL = "wss://$(API_URL):$(API_PORT)$(WS_PATH)"
SERVER_URL = "$(API_URL):$(API_PORT)"
STATIC_PATH = static

.PHONY: backend deploy frontend run

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

backend:
	WS_PATH=$(WS_PATH) \
	SERVER_URL=$(SERVER_URL) \
	STATIC_PATH=$(STATIC_PATH) \
	RUST_LOG=actix_web=info,webapp=trace \
	cargo run \
		$(BACKEND_ARGS) \
		--bin backend

deploy:
	# Deploy the frontend
	WS_URL=$(WS_URL) \
	cargo web deploy $(FRONTENT_ARGS)
	# Fix applications path to JavaScript file
	mkdir target/deploy/js
	mv target/deploy/app.js ./target/deploy/js
	# Build the docker image for static linking
	if [[ "$(shell docker images -q webapp-build:latest 2> /dev/null)" == "" ]]; then \
		docker build -f Dockerfile.build -t webapp-build . ;\
	fi
	# Build the backend
	docker run --rm -it -v $(PWD):/home/rust/src \
		-e WS_PATH=$(WS_PATH) \
		-e SERVER_URL=$(SERVER_URL) \
		-e STATIC_PATH=$(STATIC_PATH) \
		webapp-build \
		cargo build \
			$(BACKEND_ARGS) \
			--bin backend
	# Create the docker image from the executable
	docker build --no-cache \
		-f Dockerfile.webapp \
		--build-arg STATIC_PATH=$(STATIC_PATH) \
		--build-arg API_PORT=$(API_PORT) \
		-t webapp .

run:
	docker run \
		--hostname $(API_URL) \
		-p $(API_PORT):$(API_PORT) \
		-it webapp
