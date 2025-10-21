use crate::scheme::{Scheme, SchemeCow, SchemeRef};

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
		match self {
			Scheme::Regular => SchemeRef::Regular,
			Scheme::Search(d) => SchemeRef::Search(d),
			Scheme::Archive(d) => SchemeRef::Archive(d),
			Scheme::Sftp(d) => SchemeRef::Sftp(d),
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
	fn kind(&self) -> &'static str { self.as_scheme().kind() }

	fn domain(&self) -> Option<&str> { self.as_scheme().domain() }

	fn covariant(&self, other: impl AsScheme) -> bool { self.as_scheme().covariant(other) }

	fn is_local(&self) -> bool { self.as_scheme().is_local() }

	fn is_remote(&self) -> bool { self.as_scheme().is_remote() }

	fn is_virtual(&self) -> bool { self.as_scheme().is_virtual() }
}

impl SchemeLike for Scheme {}
impl SchemeLike for SchemeCow<'_> {}
