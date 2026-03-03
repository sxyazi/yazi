use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[inline]
pub fn timestamp_us() -> u64 {
	SystemTime::now().duration_since(UNIX_EPOCH).expect("Time went backwards").as_micros() as _
}

pub fn format_duration(duration: Duration) -> String {
	// go till days precision, but only show the non-zero parts
	let mut secs = duration.as_secs();
	let days = secs / 86_400;
	secs %= 86_400;
	let hours = secs / 3_600;
	secs %= 3_600;
	let minutes = secs / 60;
	secs %= 60;
	let mut parts = Vec::with_capacity(4);
	if days > 0 {
		parts.push(format!("{}d", days));
	}
	if hours > 0 || days > 0 {
		parts.push(format!("{:0>2}h", hours));
	}
	if minutes > 0 || hours > 0 || days > 0 {
		parts.push(format!("{:0>2}m", minutes));
	}
	if secs > 0 || minutes > 0 || hours > 0 || days > 0 {
		parts.push(format!("{:0>2}s", secs));
	}
	if parts.is_empty() {
		"0s".to_string()
	} else {
		parts.join("")
	}
}