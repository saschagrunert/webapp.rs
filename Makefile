# Compiler configuration
GENERAL_ARGS = --release
FRONTEND_ARGS = $(GENERAL_ARGS)
BACKEND_ARGS = $(GENERAL_ARGS) -p webapp-backend
CONTAINER_RUNTIME ?= podman

# Application configuration
define get_config_value
	$(shell sed -ne 's/^$(1).*"\(.*\)"/\1/p' Config.toml)
endef

API_URL := $(strip $(call get_config_value,url))
PG_HOST := $(strip $(call get_config_value,host))
PG_USERNAME := $(strip $(call get_config_value,username))
PG_PASSWORD := $(strip $(call get_config_value,password))
PG_DATABASE := $(strip $(call get_config_value,database))

.PHONY: \
	build-backend \
	build-doc \
	build-frontend \
	coverage \
	deploy \
	lint-rustfmt \
	lint-clippy \
	run-app \
	run-backend \
	run-frontend \
	run-postgres \
	stop-app \
	stop-postgres \
	test-deploy

ifndef VERBOSE
.SILENT:
else
GENERAL_ARGS += -v
endif

all: build-backend build-frontend

build-backend:
	cargo build $(BACKEND_ARGS)

build-doc:
	cargo doc --all --no-deps

build-frontend:
	cd frontend && \
		wasm-pack build --target web $(FRONTEND_ARGS) && \
		rollup ./main.js --format iife --file ./pkg/webapp_frontend.js

coverage:
	cd backend && cargo kcov

deploy:
	# Deploy the frontend
	sudo chown -R $(USER) .
	$(CONTAINER_RUNTIME) pull saschagrunert/build-rust:latest
	$(CONTAINER_RUNTIME) run --rm -it -w /deploy -v $(shell pwd):/deploy \
		saschagrunert/build-rust:latest \
		make build-frontend
	sudo chown -R $(USER) .
	# Build the backend
	sudo chown -R 1000:1000 .
	sudo chmod -R 777 .
	$(CONTAINER_RUNTIME) pull ekidd/rust-musl-builder:latest
	$(CONTAINER_RUNTIME) run --rm -it -v $(shell pwd):/home/rust/src \
		ekidd/rust-musl-builder:latest \
		cargo build $(BACKEND_ARGS)
	# Create the container image from the executable
	$(CONTAINER_RUNTIME) build --no-cache -t webapp .

lint-clippy:
	cargo clippy --all -- -D warnings

lint-rustfmt:
	cargo fmt
	git diff --exit-code

run-app: run-postgres
	if [ ! "$(shell $(CONTAINER_RUNTIME) ps -q -f name=webapp)" ]; then \
		$(CONTAINER_RUNTIME) run --rm \
			--name webapp \
			--network="host" \
			-v $(shell pwd)/backend/tls:/tls \
			-v $(shell pwd)/Config.toml:/Config.toml \
			-d webapp ;\
	else \
		echo "App already running" ;\
	fi

run-backend: run-postgres
	cargo run $(BACKEND_ARGS)

run-frontend: build-frontend
	cd frontend && python3 -m http.server 8000

run-postgres:
	if [ ! "$(shell $(CONTAINER_RUNTIME) ps -q -f name=postgres)" ]; then \
		$(CONTAINER_RUNTIME) run --rm --name postgres \
			-e POSTGRES_USER=$(PG_USERNAME) \
			-e POSTGRES_PASSWORD=$(PG_PASSWORD) \
			-e POSTGRES_DB=$(PG_DATABASE) \
			-p 5432:5432 \
			-d postgres ;\
		while true; do \
			if pg_isready -qh $(PG_HOST); then break; fi \
		done ;\
		sleep 1; \
		diesel migration run --database-url \
			postgres://$(PG_USERNAME):$(PG_PASSWORD)@$(PG_HOST)/$(PG_DATABASE) ;\
	else \
		echo "Database already running" ;\
	fi

stop-app: stop-postgres
	$(CONTAINER_RUNTIME) stop webapp

stop-postgres:
	$(CONTAINER_RUNTIME) stop postgres

test-deploy: run-app
	echo "Testing $(API_URL)"
	RESPONSE_CODE=$(shell curl -sw %{http_code} -o /dev/null $(API_URL)/index.html) &&\
	if [ $$RESPONSE_CODE -ne 200 ]; then \
		echo "Error: Wrong response code: $$RESPONSE_CODE" ;\
		curl -v $(API_URL) ;\
		exit 1 ;\
	else \
		echo "Got correct response code 200" ;\
	fi
