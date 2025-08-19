use std::{borrow::Borrow, hash::{Hash, Hasher}, ops::Deref, ptr::NonNull};

use hashbrown::Equivalent;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct SymbolPtr(NonNull<[u8]>);

unsafe impl Send for SymbolPtr {}

unsafe impl Sync for SymbolPtr {}

impl Deref for SymbolPtr {
	type Target = NonNull<[u8]>;

	fn deref(&self) -> &Self::Target { &self.0 }
}

impl Borrow<[u8]> for SymbolPtr {
	fn borrow(&self) -> &[u8] { self.bytes() }
}

impl Hash for SymbolPtr {
	fn hash<H: Hasher>(&self, state: &mut H) { self.bytes().hash(state); }
}

impl Equivalent<[u8]> for SymbolPtr {
	fn equivalent(&self, key: &[u8]) -> bool { self.bytes() == key }
}

impl SymbolPtr {
	#[inline]
	pub(super) fn leaked(leaked: &'static mut [u8]) -> Self { Self(NonNull::from(leaked)) }

	#[inline]
	pub(super) fn bytes(&self) -> &[u8] { unsafe { self.0.as_ref() } }
}
