# Compiler configuration
GENERAL_ARGS = --release
FRONTEND_TARGET = $(GENERAL_ARGS) --target wasm32-unknown-unknown
FRONTENT_ARGS = $(FRONTEND_TARGET) --no-default-features --features=frontend
BACKEND_TARGET = $(GENERAL_ARGS)
BACKEND_ARGS = $(BACKEND_TARGET)

# Application configuration
define get_config_value
	$(shell sed -ne 's/^$(1).*"\(.*\)"/\1/p' Config.toml)
endef

API_HOST := $(strip $(call get_config_value,ip))
API_PORT := $(strip $(call get_config_value,port))
PG_HOST := $(strip $(call get_config_value,host))
PG_USERNAME := $(strip $(call get_config_value,username))
PG_PASSWORD := $(strip $(call get_config_value,password))
PG_DATABASE := $(strip $(call get_config_value,database))

.PHONY: backend deploy frontend run startdb stopdb

ifndef VERBOSE
.SILENT:
else
GENERAL_ARGS += -v
endif

backend: startdb
	cargo run $(BACKEND_ARGS) --bin backend

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
		--build-arg API_PORT=$(API_PORT) \
		-f Dockerfile.webapp \
		-t webapp .

frontend:
	cargo web start $(FRONTENT_ARGS) --auto-reload --host 0.0.0.0

run: startdb
	docker run --rm \
		--name webapp \
		--network="host" \
		-d webapp

startdb:
	if [ ! "$(shell docker ps -q -f name=postgres)" ]; then \
		docker run --rm --name postgres \
			-e POSTGRES_USER=$(PG_USERNAME) \
			-e POSTGRES_PASSWORD=$(PG_PASSWORD) \
			-e POSTGRES_DB=$(PG_DATABASE) \
			-p 5432:5432 \
			-d postgres ;\
		sleep 5 ;\
		diesel migration run --database-url \
			postgres://$(PG_USERNAME):$(PG_PASSWORD)@$(PG_HOST)/$(PG_DATABASE) ;\
	fi

stopdb:
	docker stop postgres
