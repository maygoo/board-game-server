pub mod tic_tac_toe;

pub type ChannelBuf = Vec<u8>;

use std::time::Duration;
pub const WAIT_MS: u32 = 100;
pub const WAIT: Duration = Duration::from_millis(WAIT_MS as u64);