# Board Games with Rust

Small project to create a tcp server to facilitate online gameplay of various to-be-implemented board games.

## Layout

The `server` binary handles incoming tcp connections. Clients connect to this server to find other clients/players and match up to play the games. This server is multithreaded and currently runs on an ec2 instance.

The `client` binary handles the ux, with the player using this executable to connect to the server. This is currently a cli application but the goal is to move this to a gui using wasm and host it on a webpage.

The `common` library is used by both server and clients to define the standard message structs and game logic for the various games.
