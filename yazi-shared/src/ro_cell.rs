use std::{cell::UnsafeCell, fmt::{self, Display}, mem, ops::Deref};

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
		debug_assert!(!self.initialized());
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

	#[inline]
	pub fn replace(&self, value: T) -> T {
		debug_assert!(self.initialized());
		unsafe { mem::replace(&mut *self.0.get(), Some(value)).unwrap_unchecked() }
	}

	#[inline]
	pub fn drop(&self) -> T {
		debug_assert!(self.initialized());
		unsafe { mem::take(&mut *self.0.get()).unwrap_unchecked() }
	}

	#[inline]
	fn initialized(&self) -> bool { unsafe { (*self.0.get()).is_some() } }
}

impl<T> Deref for RoCell<T> {
	type Target = T;

	fn deref(&self) -> &Self::Target {
		debug_assert!(self.initialized());
		unsafe { (*self.0.get()).as_ref().unwrap_unchecked() }
	}
}

impl<T> Display for RoCell<T>
where
	T: Display,
{
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { self.deref().fmt(f) }
}
