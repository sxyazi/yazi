use std::{fmt, ops::Deref};

use super::{Auth, AuthArc, AuthKind};

impl fmt::Display for Auth {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { Display(self).fmt(f) }
}

impl fmt::Display for AuthArc {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { self.deref().fmt(f) }
}

// --- Display
pub struct Display<'a>(pub(crate) &'a Auth);

impl Deref for Display<'_> {
	type Target = Auth;

	fn deref(&self) -> &Self::Target { self.0 }
}

impl fmt::Display for Display<'_> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self.kind {
			AuthKind::Regular => Ok(()),
			_ => write!(f, "{}://{}", self.scheme, self.domain),
		}
	}
}

// --- DisplayPrefix
pub(crate) struct DisplayPrefix<'a>(pub(crate) &'a Auth);

impl Deref for DisplayPrefix<'_> {
	type Target = Auth;

	fn deref(&self) -> &Self::Target { self.0 }
}

impl fmt::Display for DisplayPrefix<'_> {
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
					auth.domain.fmt(f)?;
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
