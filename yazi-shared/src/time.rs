use std::time::{self, SystemTime};

pub fn timestamp_ms() -> u64 {
	SystemTime::now().duration_since(time::UNIX_EPOCH).unwrap().as_millis() as u64
}
