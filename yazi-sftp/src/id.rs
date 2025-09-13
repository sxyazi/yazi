use std::sync::atomic::{AtomicU32, Ordering};

pub(super) struct Id(AtomicU32);

impl Default for Id {
	fn default() -> Self { Self(AtomicU32::new(1)) }
}

impl Id {
	pub(super) fn next(&self) -> u32 {
		loop {
			let old = self.0.fetch_add(1, Ordering::Relaxed);
			if old != 0 {
				return old;
			}
		}
	}
}
