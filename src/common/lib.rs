pub mod tic_tac_toe;

pub type ChannelBuf = Vec<u8>;

use std::time::Duration;

/// 100 milliseconds
pub const WAIT_MS: u32 = 100;
/// Defines a standard wait time for threads to sleep
/// while waiting for other threads to unlock shared
/// resources. Value defined in ms by [`WAIT_MS`].
pub const WAIT: Duration = Duration::from_millis(WAIT_MS as u64);

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