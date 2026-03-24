FROM rust:latest AS builder

RUN rustup target add wasm32-unknown-unknown && \
    cargo install cargo-leptos

WORKDIR /app
COPY . .
RUN cargo leptos build --release

FROM debian:bookworm-slim

RUN apt-get update && \
    apt-get install -y --no-install-recommends ca-certificates && \
    rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/webapp /usr/local/bin/webapp
COPY --from=builder /app/target/site /app/site

WORKDIR /app
ENV LEPTOS_SITE_ROOT=site
ENV LEPTOS_SITE_ADDR=0.0.0.0:3000

EXPOSE 3000
CMD ["webapp"]
