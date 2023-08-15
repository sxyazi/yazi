use std::{cell::UnsafeCell, ops::Deref};

// Read-only cell. It's safe to use this in a static variable, but it's not safe
// to mutate it. This is useful for storing static data that is expensive to
// initialize, but is immutable once.
pub struct RoCell<T>(UnsafeCell<Option<T>>);

unsafe impl<T> Sync for RoCell<T> {}

impl<T> RoCell<T> {
	#[inline]
	pub const fn new() -> Self { Self(UnsafeCell::new(None)) }

	#[inline]
	pub fn init(&self, value: T) {
		unsafe {
			*self.0.get() = Some(value);
		}
	}

	#[inline]
	pub fn with<F>(&self, f: F)
	where
		F: FnOnce() -> T,
	{
		self.init(f());
	}
}

impl<T> Deref for RoCell<T> {
	type Target = T;

	fn deref(&self) -> &Self::Target { unsafe { (*self.0.get()).as_ref().unwrap() } }
}
