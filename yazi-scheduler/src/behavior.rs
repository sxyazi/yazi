use std::sync::atomic::{AtomicU64, Ordering};

use yazi_shared::Id;

pub struct Behavior {
	first_id: AtomicU64,
}

impl Behavior {
	pub(super) fn new() -> Self { Self { first_id: AtomicU64::new(0) } }

	pub(super) fn update(&self, id: Id) {
		self.first_id.compare_exchange(0, id.get(), Ordering::Relaxed, Ordering::Relaxed).ok();
	}

	pub fn reset(&self) { self.first_id.store(0, Ordering::Relaxed); }

	pub fn first_id(&self) -> Id { self.first_id.load(Ordering::Relaxed).into() }
}
