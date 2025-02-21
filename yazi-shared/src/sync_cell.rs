use std::{cell::Cell, fmt::{Debug, Display, Formatter}, ops::Deref};

/// [`SyncCell`], but [`Sync`].
///
/// This is just an `Cell`, except it implements `Sync`
/// if `T` implements `Sync`.
pub struct SyncCell<T: ?Sized>(Cell<T>);

unsafe impl<T: ?Sized + Sync> Sync for SyncCell<T> {}

impl<T> SyncCell<T> {
	#[inline]
	pub const fn new(value: T) -> Self { Self(Cell::new(value)) }
}

impl<T: Default> Default for SyncCell<T> {
	fn default() -> Self { Self::new(T::default()) }
}

impl<T> Deref for SyncCell<T> {
	type Target = Cell<T>;

	#[inline]
	fn deref(&self) -> &Self::Target { &self.0 }
}

impl<T: Copy> Clone for SyncCell<T> {
	#[inline]
	fn clone(&self) -> SyncCell<T> { SyncCell::new(self.get()) }
}

impl<T: Copy + Debug> Debug for SyncCell<T> {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result { Debug::fmt(&self.get(), f) }
}

impl<T: Copy + Display> Display for SyncCell<T> {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result { Display::fmt(&self.get(), f) }
}
