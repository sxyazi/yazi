use crate::{pool::InternStr, scheme::{AsScheme, Scheme}};

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum SchemeRef<'a> {
	#[default]
	Regular,

	Search(&'a str),

	Archive(&'a str),

	Sftp(&'a str),
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
	pub fn covariant(self, other: impl AsScheme) -> bool {
		let other = other.as_scheme();
		if self.is_virtual() || other.is_virtual() { self == other } else { true }
	}

	#[inline]
	pub fn is_local(self) -> bool {
		match self {
			Self::Regular | Self::Search(_) => true,
			Self::Archive(_) | Self::Sftp(_) => false,
		}
	}

	#[inline]
	pub fn is_remote(self) -> bool {
		match self {
			Self::Regular | Self::Search(_) | Self::Archive(_) => false,
			Self::Sftp(_) => true,
		}
	}

	#[inline]
	pub fn is_virtual(self) -> bool {
		match self {
			Self::Regular | Self::Search(_) => false,
			Self::Archive(_) | Self::Sftp(_) => true,
		}
	}
}
