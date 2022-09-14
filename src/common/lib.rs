pub mod tic_tac_toe;

pub type ChannelBuf = Vec<u8>;

use std::time::Duration;

/// 100 milliseconds
pub const WAIT_MS: u32 = 100;
/// Defines a standard wait time for threads to sleep
/// while waiting for other threads to unlock shared
/// resources. Value defined in ms by `common::WAIT_MS`.
pub const WAIT: Duration = Duration::from_millis(WAIT_MS as u64);