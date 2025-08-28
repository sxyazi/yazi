use std::hash::{Hash, Hasher};

use crate::{pool::Symbol, scheme::SchemeRef};

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub enum Scheme {
	#[default]
	Regular,

	Search(Symbol<str>),

	Archive(Symbol<str>),

	Sftp(Symbol<str>),
}

impl Hash for Scheme {
	fn hash<H: Hasher>(&self, state: &mut H) { self.as_ref().hash(state); }
}

impl Scheme {
	#[inline]
	pub fn as_ref(&self) -> SchemeRef<'_> { self.into() }

	#[inline]
	pub fn kind(&self) -> &'static str { self.as_ref().kind() }

	#[inline]
	pub fn domain(&self) -> Option<&str> { self.as_ref().domain() }

	#[inline]
	pub fn covariant(&self, other: &Self) -> bool { self.as_ref().covariant(other) }

	#[inline]
	pub fn is_virtual(&self) -> bool { self.as_ref().is_virtual() }
}
