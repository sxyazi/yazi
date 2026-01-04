use crate::scheme::{Scheme, SchemeCow, SchemeKind, SchemeRef};

pub trait AsScheme {
	fn as_scheme(&self) -> SchemeRef<'_>;
}

impl AsScheme for SchemeRef<'_> {
	#[inline]
	fn as_scheme(&self) -> SchemeRef<'_> { *self }
}

impl AsScheme for Scheme {
	#[inline]
	fn as_scheme(&self) -> SchemeRef<'_> {
		match *self {
			Self::Regular { uri, urn } => SchemeRef::Regular { uri, urn },
			Self::Search { ref domain, uri, urn } => SchemeRef::Search { domain, uri, urn },
			Self::Archive { ref domain, uri, urn } => SchemeRef::Archive { domain, uri, urn },
			Self::Sftp { ref domain, uri, urn } => SchemeRef::Sftp { domain, uri, urn },
		}
	}
}

impl AsScheme for &Scheme {
	#[inline]
	fn as_scheme(&self) -> SchemeRef<'_> { (**self).as_scheme() }
}

impl AsScheme for SchemeCow<'_> {
	#[inline]
	fn as_scheme(&self) -> SchemeRef<'_> {
		match self {
			SchemeCow::Borrowed(s) => *s,
			SchemeCow::Owned(s) => s.as_scheme(),
		}
	}
}

impl AsScheme for &SchemeCow<'_> {
	#[inline]
	fn as_scheme(&self) -> SchemeRef<'_> { (**self).as_scheme() }
}

// --- SchemeLike
pub trait SchemeLike
where
	Self: AsScheme + Sized,
{
	fn kind(&self) -> SchemeKind { *self.as_scheme() }

	fn domain(&self) -> Option<&str> { self.as_scheme().domain() }

	fn covariant(&self, other: impl AsScheme) -> bool { self.as_scheme().covariant(other) }

	fn is_local(&self) -> bool { self.as_scheme().is_local() }

	fn is_remote(&self) -> bool { self.as_scheme().is_remote() }

	fn is_virtual(&self) -> bool { self.as_scheme().is_virtual() }
}

impl SchemeLike for Scheme {}
impl SchemeLike for SchemeCow<'_> {}
