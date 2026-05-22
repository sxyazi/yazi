use std::time::{Duration, Instant};

#[derive(Clone, Copy, Debug)]
pub(crate) struct Timeout {
	timeout: Option<Duration>,
	start:   Instant,
}

impl Timeout {
	pub(crate) fn new(timeout: Option<Duration>) -> Self { Self { timeout, start: Instant::now() } }

	pub(crate) fn elapsed(&self) -> bool {
		self.timeout.map(|timeout| self.start.elapsed() >= timeout).unwrap_or(false)
	}

	pub(crate) fn leftover(&self) -> Option<Duration> {
		self.timeout.map(|timeout| {
			let elapsed = self.start.elapsed();
			timeout.checked_sub(elapsed).unwrap_or(Duration::ZERO)
		})
	}
}
