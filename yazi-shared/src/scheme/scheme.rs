use std::hash::{Hash, Hasher};

use crate::{pool::Symbol, scheme::{AsScheme, SchemeRef}};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Scheme {
	Regular { uri: usize, urn: usize },
	Search { domain: Symbol<str>, uri: usize, urn: usize },
	Archive { domain: Symbol<str>, uri: usize, urn: usize },
	Sftp { domain: Symbol<str>, uri: usize, urn: usize },
}

impl Hash for Scheme {
	fn hash<H: Hasher>(&self, state: &mut H) { self.as_scheme().hash(state); }
}

impl PartialEq<SchemeRef<'_>> for Scheme {
	fn eq(&self, other: &SchemeRef<'_>) -> bool { self.as_scheme() == *other }
}

impl Scheme {
	#[inline]
	pub fn into_domain(self) -> Option<Symbol<str>> {
		match self {
			Self::Regular { .. } => None,
			Self::Search { domain, .. } | Self::Archive { domain, .. } | Self::Sftp { domain, .. } => {
				Some(domain)
			}
		}
	}

	#[inline]
	pub fn with_ports(self, uri: usize, urn: usize) -> Self {
		match self {
			Self::Regular { .. } => Self::Regular { uri, urn },
			Self::Search { domain, .. } => Self::Search { domain, uri, urn },
			Self::Archive { domain, .. } => Self::Archive { domain, uri, urn },
			Self::Sftp { domain, .. } => Self::Sftp { domain, uri, urn },
		}
	}

	#[inline]
	pub fn zeroed(self) -> Self { self.with_ports(0, 0) }
}
