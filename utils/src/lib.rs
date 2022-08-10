#![cfg(target_arch = "wasm32")]

pub mod identity;
pub mod ipfs;
pub mod web3;

/// Unix time in total number of seconds to date time string.
pub fn timestamp_to_datetime(seconds: i64) -> String {
    use chrono::{DateTime, Local, TimeZone, Utc};

    let d_t_unix = Utc.timestamp(seconds, 0);

    let local_d_t = DateTime::<Local>::from(d_t_unix);

    local_d_t.format("%Y-%m-%d %H:%M:%S").to_string()
}
