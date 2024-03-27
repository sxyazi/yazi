use std::time::{self, SystemTime};

pub fn timestamp_us() -> u64 {
	SystemTime::now().duration_since(time::UNIX_EPOCH).unwrap().as_micros() as u64
}
