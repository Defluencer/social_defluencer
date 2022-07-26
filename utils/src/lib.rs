#![cfg(target_arch = "wasm32")]

pub mod r#async;
pub mod commentary;
pub mod defluencer;
pub mod identity;
pub mod ipfs;
pub mod subscriptions;
pub mod web3;
pub mod web_crypto;

use chrono::{DateTime, Local, TimeZone, Utc};

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
    let d_t_unix = match Utc.timestamp_opt(seconds, 0) {
        chrono::LocalResult::None => return String::new(),
        chrono::LocalResult::Single(time) => time,
        chrono::LocalResult::Ambiguous(_, _) => return String::new(),
    };

    let local_d_t = DateTime::<Local>::from(d_t_unix);

    local_d_t.format("%Y-%m-%d %H:%M:%S").to_string()
}

/// Take 20 bytes in hexa and prefix it with 0x
pub fn display_address(addr: [u8; 20]) -> String {
    let mut addr = hex::encode(addr);
    addr.insert_str(0, "0x");
    addr
}
