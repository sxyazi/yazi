use std::time::{SystemTime, UNIX_EPOCH};

#[inline]
pub fn timestamp_us() -> u64 {
	SystemTime::now().duration_since(UNIX_EPOCH).expect("Time went backwards").as_micros() as _
}
