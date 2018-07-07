# WebApp.rs
[![Build Status](https://travis-ci.org/saschagrunert/webapp.rs.svg)](https://travis-ci.org/saschagrunert/webapp.rs) [![License MIT](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/saschagrunert/webapp.rs/blob/master/LICENSE) [![Crates.io](https://img.shields.io/crates/v/webapp.svg)](https://crates.io/crates/webapp)
## A web application completely written in Rust
Target of this project is to write a complete web application including backend
and frontend within Rust.

## Build
The following build dependencies needs to be fulfilled to support the full
feature set of this application:

- [cargo-web](https://github.com/koute/cargo-web)
- [capnproto](https://github.com/capnproto/capnproto)
- [diesel_cli](https://github.com/diesel-rs/diesel)
- [docker](https://github.com/docker/docker-ce)
- [postgresql (libpg)](https://www.postgresql.org/)

The app consist of a frontend and a backend. For getting started with hacking,
the backend can tested via `make backend`, whereas the frontend can be
tested with `make frontend`. You can adapt the application configuration
within `Config.toml` if needed.

## Run
If both, the backend and frontend are running, you can visit the web application
at [`http://127.0.0.1:8000`](http://127.0.0.1:8000). After the successful
loading of the application you should see an authentication screen like this:

![authentication screen](.github/authentication_screen.png "Authentication Screen")

Now you are able to login with a matching username and password combination like
`me` (username) and `me` (password). There is currently no further user
authentication yet, but non matching combination will result in an
authentication failure. After the successfully login you should be able to see
the content of the application:

![content screen](.github/content_screen.png "Content Screen")

The authentication should persist, even after a manual page reload. Logging out
of the application via the logout button should also work as intended.

### Control Flow
The complete control flow of the application looks like this:

![control screen](.github/flow_chart.png "Control Flow")

## Deploy
To deploy the application as a docker image, simply run:

```console
make deploy
```

After that you can run the application side by side with a PostgreSQL container
via:

```console
make run
```

The application should now be accesible at
[`http://127.0.0.1:30080`](http://127.0.0.1:30080).

## Contributing
You want to contribute to this project? Wow, thanks! So please just fork it and
send me a pull request.
