pub mod tic_tac_toe;

pub type ChannelBuf = Vec<u8>;

use serde::{Serialize, Deserialize};
use std::time::Duration;

/// Wait time in milliseconds.
const THREAD_SLEEP_MS: u64 = 100;

/// Defines a standard wait time for threads to sleep
/// while waiting for other threads to unlock shared
/// resources. Value defined in ms by [`THREAD_SLEEP_MS`].
pub const THREAD_SLEEP: Duration = Duration::from_millis(THREAD_SLEEP_MS);

const PING_INTERVAL_MS: u64 = 1000;

/// Defines the ratio between the thread sleep and ping intervals.
pub const PING_INTERVAL: u64 = PING_INTERVAL_MS / THREAD_SLEEP_MS;

/// Remote IP for the client to connect to by default.
/// 
/// Changes from `127.0.0.1` for debug builds and
/// `ws.gh.maygoo.au` for `--release` builds.
#[cfg(debug_assertions)]
pub const REMOTE_IP: &str = "127.0.0.1";
#[cfg(not(debug_assertions))]
pub const REMOTE_IP: &str = "ws.gh.maygoo.au";

/// Remote port for the server to use and the client
/// to connect to by default.
pub const REMOTE_PORT: u16 = 3334;

/// Server messages, indiscriminate of the selected game.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ServerMessage {
    Ping(ServerStatus),
}

/// Server status sent to each client.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ServerStatus {
    pub n_players: usize,
}
