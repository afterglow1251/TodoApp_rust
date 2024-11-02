use std::time::{SystemTime, UNIX_EPOCH};

pub fn get_current_timestamp() -> usize {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as usize
}
