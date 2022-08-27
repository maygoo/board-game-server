# Board Games with Rust

[![Rust](https://github.com/maygoo/board-games-rust/actions/workflows/rust.yml/badge.svg)](https://github.com/maygoo/board-games-rust/actions/workflows/rust.yml)

Small project to create a tcp server to facilitate online gameplay of various board games. Currently there is a working implementation for tic tac toe for any square board size.

## Crate Structure

The `server` binary handles incoming tcp connections. Clients connect to this server to find other clients/players and match up to play the games. This server is multithreaded and is designed to be non-blocking to handle any number of incomming connections.

The `client` binary handles the ux, with the player using this executable to connect to the server. This is currently a cli application but the goal is to move this to a gui using wasm and host it on a webpage.

The `common` library is used by both server and clients to define the standard message structs and game logic for the various games.

## Running

To run an instance of the server:

```bash
cargo run --bin server --release
```

To run an instance of the client:

```bash
cargo run --bin client --release
```

Omit the `--release` flag when running the client if you want to connect to a local server, otherwise the client will connect to my instance of the server which is currently running on an ec2 instance.

## How to Play

Currently the server waits until it has two clients then it pairs them together automatically and they start a game of tic tac toe with a board size of 3.
