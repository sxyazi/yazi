use std::sync::atomic::{AtomicU64, Ordering};

use crate::id::Id;

#[derive(Debug)]
pub struct Ids {
	next: AtomicU64,
}

impl Ids {
	#[inline]
	pub const fn new() -> Self { Self { next: AtomicU64::new(1) } }

	#[inline]
	pub fn next(&self) -> Id {
		loop {
			let old = self.next.fetch_add(1, Ordering::Relaxed);
			if old != 0 {
				return Id(old);
			}
		}
	}

	#[inline]
	pub fn current(&self) -> Id { Id(self.next.load(Ordering::Relaxed)) }
}

impl Default for Ids {
	fn default() -> Self { Self::new() }
}
