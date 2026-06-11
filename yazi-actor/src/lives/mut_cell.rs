use std::cell::UnsafeCell;

pub(super) struct MutCell<T>(UnsafeCell<T>);

unsafe impl<T> Sync for MutCell<T> {}

impl<T> MutCell<T> {
	pub(super) const fn new(value: T) -> Self { Self(UnsafeCell::new(value)) }

	pub(super) fn get(&self) -> *mut T { self.0.get() }
}
