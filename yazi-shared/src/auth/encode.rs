use std::fmt;

use percent_encoding::{AsciiSet, CONTROLS, PercentEncode, percent_encode};

use super::{Auth, AuthKind, Domain};

pub struct EncodeAuth<'a>(pub &'a Auth, pub bool);

impl EncodeAuth<'_> {
	pub fn domain<'a>(s: &'a Domain<'_>) -> PercentEncode<'a> {
		const SET: &AsciiSet = &CONTROLS.add(b'/').add(b':').add(b'%');
		percent_encode(s.as_bytes(), SET)
	}
}

impl fmt::Display for EncodeAuth<'_> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(
			f,
			"{}{}://{}",
			self.0.scheme,
			if self.1 { "~" } else { "" },
			Self::domain(&self.0.domain)
		)
	}
}

// --- EncodePrefix
pub struct EncodePrefix<'a>(pub &'a Auth);

impl EncodePrefix<'_> {
	pub fn parent<'a>(s: &'a Domain<'_>) -> PercentEncode<'a> {
		const SET: &AsciiSet = &CONTROLS.add(b'/').add(b',').add(b'@').add(b'%');
		percent_encode(s.as_bytes(), SET)
	}
}

impl fmt::Display for EncodePrefix<'_> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		if self.0.kind != AuthKind::Hub {
			return f.write_str("/");
		}

		f.write_str("/@")?;
		let (mut first, mut parent) = (true, self.0.parent.as_deref());
		while let Some(auth) = parent {
			if !first {
				f.write_str(",")?;
			}
			Self::parent(&auth.domain).fmt(f)?;
			(first, parent) = (false, auth.parent.as_deref());
		}
		f.write_str("/")
	}
}
