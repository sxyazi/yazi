use crate::{pool::InternStr, scheme::Scheme};

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum SchemeRef<'a> {
	#[default]
	Regular,

	Search(&'a str),

	Archive(&'a str),

	Sftp(&'a str),
}

impl<'a> From<&'a Scheme> for SchemeRef<'a> {
	fn from(value: &'a Scheme) -> Self {
		match value {
			Scheme::Regular => Self::Regular,
			Scheme::Search(d) => Self::Search(d),
			Scheme::Archive(d) => Self::Archive(d),
			Scheme::Sftp(d) => Self::Sftp(d),
		}
	}
}

impl From<SchemeRef<'_>> for Scheme {
	fn from(value: SchemeRef) -> Self {
		match value {
			SchemeRef::Regular => Self::Regular,
			SchemeRef::Search(d) => Self::Search(d.intern()),
			SchemeRef::Archive(d) => Self::Archive(d.intern()),
			SchemeRef::Sftp(d) => Self::Sftp(d.intern()),
		}
	}
}

impl<'a> SchemeRef<'a> {
	#[inline]
	pub const fn kind(self) -> &'static str {
		match self {
			Self::Regular => "regular",
			Self::Search(_) => "search",
			Self::Archive(_) => "archive",
			Self::Sftp(_) => "sftp",
		}
	}

	#[inline]
	pub const fn domain(self) -> Option<&'a str> {
		match self {
			Self::Regular => None,
			Self::Search(s) | Self::Archive(s) | Self::Sftp(s) => Some(s),
		}
	}

	#[inline]
	pub fn covariant(self, other: impl Into<Self>) -> bool {
		let other = other.into();
		if self.is_virtual() || other.is_virtual() { self == other } else { true }
	}

	#[inline]
	pub fn is_virtual(&self) -> bool {
		match self {
			Self::Regular | Self::Search(_) => false,
			Self::Archive(_) | Self::Sftp(_) => true,
		}
	}
}
