# webapp.rs

[![CI](https://github.com/saschagrunert/webapp.rs/actions/workflows/ci.yml/badge.svg)](https://github.com/saschagrunert/webapp.rs/actions/workflows/ci.yml)
[![Docs](https://github.com/saschagrunert/webapp.rs/actions/workflows/gh-pages.yml/badge.svg)](https://saschagrunert.github.io/webapp.rs)
[![Coverage](https://codecov.io/gh/saschagrunert/webapp.rs/branch/main/graph/badge.svg)](https://codecov.io/gh/saschagrunert/webapp.rs)
[![Docs](https://docs.rs/webapp/badge.svg)](https://docs.rs/webapp)
[![Crates.io](https://img.shields.io/crates/v/webapp.svg)](https://crates.io/crates/webapp)
[![License Apache 2.0](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](https://github.com/saschagrunert/webapp.rs/blob/main/LICENSE)

## A web application completely written in Rust

Target of this project is to write a complete web application including backend
and frontend within Rust.

```mermaid
graph LR
    A[Leptos / WASM<br>in browser] -->|SSR / RPC| B[Axum<br>HTTP Server]
    B --> C[SQLx]
    C --> D[(PostgreSQL)]
```

### Blog Posts

1. [A Web Application completely in Rust](https://medium.com/@saschagrunert/a-web-application-completely-in-rust-6f6bdb6c4471).
2. [Lessons learned on writing web applications completely in Rust](https://medium.com/@saschagrunert/lessons-learned-on-writing-web-applications-completely-in-rust-2080d0990287).

## Architecture

| Component | Technology |
|-----------|------------|
| Frontend  | [Leptos](https://leptos.dev) (WebAssembly with SSR + hydration) |
| Backend   | [Axum](https://github.com/tokio-rs/axum) (via leptos_axum) |
| Database  | [PostgreSQL](https://www.postgresql.org) (via SQLx) |
| Auth      | JWT tokens (jsonwebtoken) + Argon2 password hashing |

The application uses Leptos server functions to communicate between frontend and
backend, eliminating the need for a separate REST API layer. Both server and
client are compiled from a single Rust crate.

## Features

- User registration with Argon2 password hashing
- Login with username and password
- JWT-based session management with automatic renewal
- PostgreSQL session and user storage
- CSRF protection via origin validation
- Health check endpoint (`/healthz`) for container orchestration
- Server-side rendering with client-side hydration
- Single binary deployment

## Prerequisites

- [Rust](https://rustup.rs) (stable)
- [cargo-leptos](https://github.com/leptos-rs/cargo-leptos): `cargo install cargo-leptos`
- [PostgreSQL](https://www.postgresql.org)
- `wasm32-unknown-unknown` target: `rustup target add wasm32-unknown-unknown`
- [wasm-bindgen-cli](https://rustwasm.github.io/wasm-bindgen/): `cargo install wasm-bindgen-cli`

## Getting Started

Start a PostgreSQL instance:

```sh
docker run -d --name postgres \
    -e POSTGRES_USER=webapp \
    -e POSTGRES_PASSWORD=webapp \
    -e POSTGRES_DB=webapp \
    -p 5432:5432 \
    postgres:17
```

Run the application:

```sh
export DATABASE_URL=postgres://webapp:webapp@localhost/webapp
cargo leptos watch
```

The application will be available at `http://127.0.0.1:3000`.

Register a new account using the "Register" link on the login page, then log in
with your credentials.

## Configuration

| Environment Variable | Description | Default |
|---------------------|-------------|---------|
| `DATABASE_URL` | PostgreSQL connection string | `postgres://localhost/webapp` |
| `JWT_SECRET` | Secret key for JWT token signing | `change-me-in-production` |
| `LEPTOS_SITE_ADDR` | Server listen address | `127.0.0.1:3000` |

## Container

Build and run as a container:

```sh
docker build -t webapp .
docker run -p 3000:3000 \
    -e DATABASE_URL=postgres://webapp:webapp@host.docker.internal/webapp \
    webapp
```

## Development

```sh
cargo fmt --check                              # Check formatting
cargo clippy --features ssr -- -D warnings     # Lint server code
cargo test --features ssr                      # Run tests
cargo leptos build                             # Build for development
cargo leptos build --release                   # Build for production
```

## Contributing

You want to contribute to this project? Wow, thanks! So please just fork it and
send me a pull request.

## License

[Apache 2.0](LICENSE)
