use std::{fmt, ops::Deref};

use super::{Auth, AuthKind};

impl Auth {
	pub(crate) fn encode(&self, tilde: bool) -> Encode<'_> { Encode(self, tilde) }
}

// --- Encode
pub struct Encode<'a>(&'a Auth, bool);

impl Deref for Encode<'_> {
	type Target = Auth;

	fn deref(&self) -> &Self::Target { self.0 }
}

impl fmt::Display for Encode<'_> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{}{}://{}", self.scheme, if self.1 { "~" } else { "" }, self.domain.encode())
	}
}

// --- EncodePrefix
pub struct EncodePrefix<'a>(pub(crate) &'a Auth);

impl Deref for EncodePrefix<'_> {
	type Target = Auth;

	fn deref(&self) -> &Self::Target { self.0 }
}

impl fmt::Display for EncodePrefix<'_> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self.kind {
			AuthKind::Regular => return Ok(()),
			AuthKind::Hub => {
				f.write_str("/@")?;
				let (mut first, mut parent) = (true, self.parent.as_ref());
				while let Some(auth) = parent {
					if !first {
						f.write_str(",")?;
					}
					auth.domain.encode_parent().fmt(f)?;
					(first, parent) = (false, auth.parent.as_ref());
				}
			}
			AuthKind::View => {
				let data = self.view.data().ok_or(fmt::Error)?;
				f.write_str("/@")?;
				data.encode(f)?;
			}
			_ => {}
		}

		f.write_str("/")
	}
}
