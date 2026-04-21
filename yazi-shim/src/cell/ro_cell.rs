use std::{cell::UnsafeCell, fmt::{self, Display}, mem::MaybeUninit, ops::Deref};

// Read-only cell. It's safe to use this in a static variable, but it's not safe
// to mutate it. This is useful for storing static data that is expensive to
// initialize, but is immutable once.
pub struct RoCell<T> {
	inner:       UnsafeCell<MaybeUninit<T>>,
	#[cfg(debug_assertions)]
	initialized: UnsafeCell<bool>,
}

unsafe impl<T> Sync for RoCell<T> {}

impl<T> RoCell<T> {
	#[inline]
	pub const fn new() -> Self {
		Self {
			inner:                                UnsafeCell::new(MaybeUninit::uninit()),
			#[cfg(debug_assertions)]
			initialized:                          UnsafeCell::new(false),
		}
	}

	#[inline]
	pub const fn new_const(value: T) -> Self {
		Self {
			inner:                                UnsafeCell::new(MaybeUninit::new(value)),
			#[cfg(debug_assertions)]
			initialized:                          UnsafeCell::new(true),
		}
	}

	#[inline]
	pub fn init(&self, value: T) {
		unsafe {
			#[cfg(debug_assertions)]
			assert!(!self.initialized.get().replace(true));
			*self.inner.get() = MaybeUninit::new(value);
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
	pub fn drop(&self) -> T {
		unsafe {
			#[cfg(debug_assertions)]
			assert!(self.initialized.get().replace(false));
			self.inner.get().replace(MaybeUninit::uninit()).assume_init()
		}
	}
}

impl<T> Default for RoCell<T> {
	fn default() -> Self { Self::new() }
}

impl<T> Deref for RoCell<T> {
	type Target = T;

	fn deref(&self) -> &Self::Target {
		unsafe {
			#[cfg(debug_assertions)]
			assert!(*self.initialized.get());
			(*self.inner.get()).assume_init_ref()
		}
	}
}

impl<T> Display for RoCell<T>
where
	T: Display,
{
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { self.deref().fmt(f) }
}
