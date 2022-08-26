#![cfg(target_arch = "wasm32")]

pub mod defluencer;
pub mod follows;
pub mod identity;
pub mod ipfs;
pub mod web3;

/// Translate total number of seconds to timecode.
pub fn seconds_to_timecode(seconds: f64) -> (u8, u8, u8) {
    let rem_seconds = seconds.round();

    let hours = (rem_seconds / 3600.0) as u8;
    let rem_seconds = rem_seconds.rem_euclid(3600.0);

    let minutes = (rem_seconds / 60.0) as u8;
    let rem_seconds = rem_seconds.rem_euclid(60.0);

    let seconds = rem_seconds as u8;

    (hours, minutes, seconds)
}

/// Unix time in total number of seconds to date time string.
pub fn timestamp_to_datetime(seconds: i64) -> String {
    use chrono::{DateTime, Local, TimeZone, Utc};

    let d_t_unix = Utc.timestamp(seconds, 0);

    let local_d_t = DateTime::<Local>::from(d_t_unix);

    local_d_t.format("%Y-%m-%d %H:%M:%S").to_string()
}
