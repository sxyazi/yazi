use std::{hash::{Hash, Hasher}, ops::Deref};

pub(super) struct PtrCell<T>(pub(super) *const T);

impl<T> Deref for PtrCell<T> {
	type Target = T;

	fn deref(&self) -> &Self::Target { unsafe { &*self.0 } }
}

impl<T> From<&T> for PtrCell<T> {
	fn from(value: &T) -> Self { Self(value) }
}

impl<T> Clone for PtrCell<T> {
	fn clone(&self) -> Self { *self }
}

impl<T> Copy for PtrCell<T> {}

impl<T> PartialEq for PtrCell<T> {
	fn eq(&self, other: &Self) -> bool { self.0.addr() == other.0.addr() }
}

impl<T> Eq for PtrCell<T> {}

impl<T> Hash for PtrCell<T> {
	fn hash<H: Hasher>(&self, state: &mut H) { state.write_usize(self.0.addr()); }
}

impl<T> PtrCell<T> {
	#[inline]
	pub(super) fn as_static(&self) -> &'static T { unsafe { &*self.0 } }
}
