use std::{hash::{Hash, Hasher}, marker::PhantomData, mem::ManuallyDrop, ops::Deref, str};

use hashbrown::hash_map::RawEntryMut;

use crate::pool::{Pool, SYMBOLS, SymbolPtr, compute_hash};

pub struct Symbol<T: ?Sized> {
	ptr:      SymbolPtr,
	_phantom: PhantomData<T>,
}

unsafe impl<T: ?Sized> Send for Symbol<T> {}

unsafe impl<T: ?Sized> Sync for Symbol<T> {}

impl<T: ?Sized> Clone for Symbol<T> {
	fn clone(&self) -> Self {
		let hash = compute_hash(&self.ptr);
		match SYMBOLS.lock().raw_entry_mut().from_key_hashed_nocheck(hash, &self.ptr) {
			RawEntryMut::Occupied(mut oe) => *oe.get_mut() += 1,
			RawEntryMut::Vacant(_) => unreachable!(),
		}
		Self::new(self.ptr.clone())
	}
}

impl<T: ?Sized> Drop for Symbol<T> {
	fn drop(&mut self) {
		let hash = compute_hash(&self.ptr);
		match SYMBOLS.lock().raw_entry_mut().from_key_hashed_nocheck(hash, &self.ptr) {
			RawEntryMut::Occupied(mut oe) => {
				let count = oe.get_mut();
				*count -= 1;

				if *count == 0 {
					oe.remove();
					drop(unsafe { Box::from_raw(self.ptr.as_ptr()) });
				}
			}
			RawEntryMut::Vacant(_) => unreachable!(),
		}
	}
}

impl AsRef<[u8]> for Symbol<[u8]> {
	fn as_ref(&self) -> &[u8] { self.ptr.bytes() }
}

impl AsRef<str> for Symbol<str> {
	fn as_ref(&self) -> &str { unsafe { str::from_utf8_unchecked(self.ptr.bytes()) } }
}

impl Deref for Symbol<[u8]> {
	type Target = [u8];

	fn deref(&self) -> &Self::Target { self.as_ref() }
}

impl Deref for Symbol<str> {
	type Target = str;

	fn deref(&self) -> &Self::Target { self.as_ref() }
}

// --- Default
impl Default for Symbol<[u8]> {
	fn default() -> Self { Pool::<[u8]>::intern(b"") }
}

impl Default for Symbol<str> {
	fn default() -> Self { Pool::<str>::intern("") }
}

// --- Eq
impl<T: ?Sized> PartialEq for Symbol<T> {
	fn eq(&self, other: &Self) -> bool { self.ptr == other.ptr }
}

impl<T: ?Sized> Eq for Symbol<T> {}

impl PartialEq<str> for Symbol<str> {
	fn eq(&self, other: &str) -> bool { self.as_ref() == other }
}

impl PartialEq<[u8]> for Symbol<[u8]> {
	fn eq(&self, other: &[u8]) -> bool { self.as_ref() == other }
}

// --- Hash
impl<T: ?Sized> Hash for Symbol<T> {
	fn hash<H: Hasher>(&self, state: &mut H) { self.ptr.as_ptr().hash(state); }
}

// --- Ord
impl Ord for Symbol<[u8]> {
	fn cmp(&self, other: &Self) -> std::cmp::Ordering { self.as_ref().cmp(other.as_ref()) }
}

impl Ord for Symbol<str> {
	fn cmp(&self, other: &Self) -> std::cmp::Ordering { self.as_ref().cmp(other.as_ref()) }
}

// --- PartialOrd
impl PartialOrd for Symbol<[u8]> {
	fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> { Some(self.cmp(other)) }
}

impl PartialOrd for Symbol<str> {
	fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> { Some(self.cmp(other)) }
}

// --- Display
impl std::fmt::Display for Symbol<str> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.as_ref())
	}
}

// --- Debug
impl std::fmt::Debug for Symbol<[u8]> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "Symbol<[u8]>({:?})", self.as_ref())
	}
}

impl std::fmt::Debug for Symbol<str> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "Symbol<str>({:?})", self.as_ref())
	}
}

impl<T: ?Sized> Symbol<T> {
	#[inline]
	pub(super) fn new(ptr: SymbolPtr) -> Self { Self { ptr, _phantom: PhantomData } }

	#[inline]
	pub(super) fn into_ptr(self) -> SymbolPtr { ManuallyDrop::new(self).ptr.clone() }
}
