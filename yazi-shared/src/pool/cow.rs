use std::ops::Deref;

use crate::pool::{Pool, Symbol};

pub enum SymbolCow<'a, T: ?Sized> {
	Borrowed(&'a T),
	Owned(Symbol<T>),
}

impl<T: ?Sized> Clone for SymbolCow<'_, T> {
	fn clone(&self) -> Self {
		match self {
			Self::Borrowed(t) => Self::Borrowed(t),
			Self::Owned(t) => Self::Owned(t.clone()),
		}
	}
}

impl AsRef<[u8]> for SymbolCow<'_, [u8]> {
	fn as_ref(&self) -> &[u8] {
		match self {
			Self::Borrowed(b) => b,
			Self::Owned(b) => b.as_ref(),
		}
	}
}

impl AsRef<str> for SymbolCow<'_, str> {
	fn as_ref(&self) -> &str {
		match self {
			Self::Borrowed(s) => s,
			Self::Owned(s) => s.as_ref(),
		}
	}
}

// --- Deref
impl Deref for SymbolCow<'_, [u8]> {
	type Target = [u8];

	fn deref(&self) -> &Self::Target { self.as_ref() }
}

impl Deref for SymbolCow<'_, str> {
	type Target = str;

	fn deref(&self) -> &Self::Target { self.as_ref() }
}

// --- From
impl<'a, T: ?Sized> From<&'a T> for SymbolCow<'a, T> {
	fn from(value: &'a T) -> Self { Self::Borrowed(value) }
}

impl<T: ?Sized> From<Symbol<T>> for SymbolCow<'_, T> {
	fn from(value: Symbol<T>) -> Self { Self::Owned(value) }
}

impl From<SymbolCow<'_, [u8]>> for Symbol<[u8]> {
	fn from(value: SymbolCow<'_, [u8]>) -> Self { value.into_owned() }
}

impl From<SymbolCow<'_, str>> for Symbol<str> {
	fn from(value: SymbolCow<'_, str>) -> Self { value.into_owned() }
}

// --- Debug
impl std::fmt::Debug for SymbolCow<'_, [u8]> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "SymbolCow<[u8]>({:?})", self.as_ref())
	}
}

impl std::fmt::Debug for SymbolCow<'_, str> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "SymbolCow<str>({:?})", self.as_ref())
	}
}

impl SymbolCow<'_, [u8]> {
	pub fn into_owned(self) -> Symbol<[u8]> {
		match self {
			Self::Borrowed(t) => Pool::<[u8]>::intern(t),
			Self::Owned(t) => t,
		}
	}
}

impl SymbolCow<'_, str> {
	pub fn into_owned(self) -> Symbol<str> {
		match self {
			Self::Borrowed(t) => Pool::<str>::intern(t),
			Self::Owned(t) => t,
		}
	}
}
