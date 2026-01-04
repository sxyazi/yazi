use std::{hash::Hash, ops::Deref};

use crate::{pool::InternStr, scheme::{AsScheme, Scheme, SchemeKind}};

#[derive(Clone, Copy, Debug)]
pub enum SchemeRef<'a> {
	Regular { uri: usize, urn: usize },
	Search { domain: &'a str, uri: usize, urn: usize },
	Archive { domain: &'a str, uri: usize, urn: usize },
	Sftp { domain: &'a str, uri: usize, urn: usize },
}

impl Deref for SchemeRef<'_> {
	type Target = SchemeKind;

	#[inline]
	fn deref(&self) -> &Self::Target {
		match self {
			Self::Regular { .. } => &SchemeKind::Regular,
			Self::Search { .. } => &SchemeKind::Search,
			Self::Archive { .. } => &SchemeKind::Archive,
			Self::Sftp { .. } => &SchemeKind::Sftp,
		}
	}
}

impl Hash for SchemeRef<'_> {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		self.kind().hash(state);
		self.domain().hash(state);
	}
}

impl PartialEq<SchemeRef<'_>> for SchemeRef<'_> {
	fn eq(&self, other: &SchemeRef) -> bool {
		self.kind() == other.kind() && self.domain() == other.domain()
	}
}

impl From<SchemeRef<'_>> for Scheme {
	fn from(value: SchemeRef) -> Self { value.to_owned() }
}

impl<'a> SchemeRef<'a> {
	#[inline]
	pub fn covariant(self, other: impl AsScheme) -> bool {
		let other = other.as_scheme();
		if self.is_virtual() || other.is_virtual() { self == other } else { true }
	}

	#[inline]
	pub const fn domain(self) -> Option<&'a str> {
		match self {
			Self::Regular { .. } => None,
			Self::Search { domain, .. } | Self::Archive { domain, .. } | Self::Sftp { domain, .. } => {
				Some(domain)
			}
		}
	}

	#[inline]
	pub const fn kind(self) -> SchemeKind {
		match self {
			Self::Regular { .. } => SchemeKind::Regular,
			Self::Search { .. } => SchemeKind::Search,
			Self::Archive { .. } => SchemeKind::Archive,
			Self::Sftp { .. } => SchemeKind::Sftp,
		}
	}

	#[inline]
	pub const fn ports(self) -> (usize, usize) {
		match self {
			Self::Regular { uri, urn } => (uri, urn),
			Self::Search { uri, urn, .. } => (uri, urn),
			Self::Archive { uri, urn, .. } => (uri, urn),
			Self::Sftp { uri, urn, .. } => (uri, urn),
		}
	}

	pub fn to_owned(self) -> Scheme {
		match self {
			Self::Regular { uri, urn } => Scheme::Regular { uri, urn },
			Self::Search { domain, uri, urn } => Scheme::Search { domain: domain.intern(), uri, urn },
			Self::Archive { domain, uri, urn } => Scheme::Archive { domain: domain.intern(), uri, urn },
			Self::Sftp { domain, uri, urn } => Scheme::Sftp { domain: domain.intern(), uri, urn },
		}
	}

	pub const fn with_ports(self, uri: usize, urn: usize) -> Self {
		match self {
			Self::Regular { .. } => Self::Regular { uri, urn },
			Self::Search { domain, .. } => Self::Search { domain, uri, urn },
			Self::Archive { domain, .. } => Self::Archive { domain, uri, urn },
			Self::Sftp { domain, .. } => Self::Sftp { domain, uri, urn },
		}
	}

	#[inline]
	pub const fn zeroed(self) -> Self { self.with_ports(0, 0) }
}
