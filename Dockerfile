FROM rust:latest AS chef

RUN rustup target add wasm32-unknown-unknown && \
    cargo install cargo-chef cargo-leptos

WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json --features ssr && \
    cargo chef cook --release --recipe-path recipe.json --target wasm32-unknown-unknown --features hydrate
COPY . .
RUN cargo leptos build --release

FROM debian:bookworm-slim

RUN apt-get update && \
    apt-get install -y --no-install-recommends ca-certificates curl && \
    rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/webapp /usr/local/bin/webapp
COPY --from=builder /app/target/site /app/site

WORKDIR /app
ENV LEPTOS_SITE_ROOT=site
ENV LEPTOS_SITE_ADDR=0.0.0.0:3000

EXPOSE 3000
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:3000/healthz || exit 1
CMD ["webapp"]
