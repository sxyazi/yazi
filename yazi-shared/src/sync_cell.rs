use std::{cell::Cell, fmt::{Debug, Display, Formatter}, ops::Deref};

use serde::{Deserialize, Deserializer, Serialize, Serializer};

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
	fn clone(&self) -> Self { Self::new(self.get()) }
}

impl<T: Copy + Debug> Debug for SyncCell<T> {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result { Debug::fmt(&self.get(), f) }
}

impl<T: Copy + Display> Display for SyncCell<T> {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result { Display::fmt(&self.get(), f) }
}

impl<T> Serialize for SyncCell<T>
where
	T: Copy + Serialize,
{
	fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
		self.0.serialize(serializer)
	}
}

impl<'de, T> Deserialize<'de> for SyncCell<T>
where
	T: Copy + Deserialize<'de>,
{
	fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
		Ok(Self::new(T::deserialize(deserializer)?))
	}
}
