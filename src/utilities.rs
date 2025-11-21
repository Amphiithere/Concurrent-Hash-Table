use std::time::{SystemTime, UNIX_EPOCH};

pub fn current_timestamp() -> u64 {
    return SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_micros() as u64;
}
