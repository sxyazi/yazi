use std::hash::{Hash, Hasher};

use crate::{pool::Symbol, scheme::{AsScheme, SchemeRef}};

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub enum Scheme {
	#[default]
	Regular,

	Search(Symbol<str>),

	Archive(Symbol<str>),

	Sftp(Symbol<str>),
}

impl Hash for Scheme {
	fn hash<H: Hasher>(&self, state: &mut H) { self.as_scheme().hash(state); }
}

impl PartialEq<SchemeRef<'_>> for Scheme {
	fn eq(&self, other: &SchemeRef<'_>) -> bool { self.as_scheme() == *other }
}
