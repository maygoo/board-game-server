# Board Games with Rust

[![Rust](https://github.com/maygoo/board-games-rust/actions/workflows/rust.yml/badge.svg)](https://github.com/maygoo/board-games-rust/actions/workflows/rust.yml)
[![Pages](https://github.com/maygoo/board-games-rust/actions/workflows/pages.yml/badge.svg)](https://maygoo.github.io/board-games-rust/)

Small project to create a websocket server to facilitate online gameplay of various board games. Currently there is a cli client which connects to the server to play tic tac toe. There is also a static web app as an alternative to the cli client, it is built using WASM and [egui](https://github.com/emilk/egui) and then deployed to [github pages](https://maygoo.github.io/board-games-rust/).

## How to Play

Visit the [website](https://maygoo.github.io/board-games-rust/) and follow the prompts.

## Crate Structure

The `server` binary handles incoming websocket connections. Clients connect to this server to find other clients/players and match up to play the games. This server is multithreaded and is designed to be non-blocking to handle any number of incomming connections.

The `client` binary handles the ux, with the player using this executable to connect to the server. This was a proof-of-concept cli application to test out the server capabilities and is not being developed anymore in favour of the web client.

The `common` library is used by both server and clients to define the standard message structs and game logic for the various games.

The `web` directory is another cargo crate which builds to a static WASM site. This both handles the websocket connections with the server as well as handles user input through a web browser. In the future this might be made to also work as a desktop application.

## Self-Hosting

If you want to connect to the server and play some games then read [here](#how-to-play). Otherwise, if you want to build and run your own instance of the server or clients then continue reading.

To run an instance of the server:

```bash
cargo run --bin server --release --features native
```

To run an instance of the cli client:

```bash
cargo run --bin client --release --features native
```

To run an instance of the web client run:

```bash
trunk serve --release
```

and visit [127.0.0.1](https://127.0.0.1:8080).

Omit the `--release` flag when running the client if you want to connect to a local server, otherwise the client will connect to my public instance of the server which is currently running on an ec2 instance.
