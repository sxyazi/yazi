use std::fmt::{self, Display};

use percent_encoding::{AsciiSet, CONTROLS, percent_encode};

use super::{Auth, AuthKind, Domain};

// --- EncodeAuth
pub struct EncodeAuth<'a>(pub &'a Auth, pub bool);

impl EncodeAuth<'_> {
	pub fn domain<'a>(s: &'a Domain<'_>) -> EncodeDomain<'a> {
		const SET: &AsciiSet = &CONTROLS.add(b'/').add(b':').add(b'%');
		EncodeDomain(s, SET)
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
	pub fn parent<'a>(s: &'a Domain<'_>) -> EncodeDomain<'a> {
		const SET: &AsciiSet = &CONTROLS.add(b'/').add(b',').add(b'@').add(b'%');
		EncodeDomain(s, SET)
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

// --- EncodeDomain
pub struct EncodeDomain<'a>(&'a [u8], &'static AsciiSet);

impl Display for EncodeDomain<'_> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		for chunk in self.0.utf8_chunks() {
			for c in chunk.valid().chars() {
				if c.is_ascii() {
					percent_encode(&[c as u8], self.1).fmt(f)?;
				} else {
					c.fmt(f)?;
				}
			}
			percent_encode(chunk.invalid(), self.1).fmt(f)?;
		}
		Ok(())
	}
}
