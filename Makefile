# Compiler configuration
GENERAL_ARGS = --release
FRONTEND_TARGET = $(GENERAL_ARGS) --target wasm32-unknown-unknown
FRONTENT_ARGS = $(FRONTEND_TARGET) --no-default-features --features=frontend
BACKEND_TARGET = $(GENERAL_ARGS)
BACKEND_ARGS = $(BACKEND_TARGET)

.PHONY: backend deploy frontend

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

backend:
	cargo run \
		$(BACKEND_ARGS) \
		--bin backend

deploy:
	# Deploy the frontend
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
		webapp-build \
		cargo build \
			$(BACKEND_ARGS) \
			--bin backend
	# Create the docker image from the executable
	docker build --no-cache \
		--build-arg API_PORT=$(shell sed -ne 's/^port.*"\(.*\)"/\1/p' Config.toml) \
		-f Dockerfile.webapp \
		-t webapp .
