use std::fmt;

use percent_encoding::{AsciiSet, CONTROLS, PercentEncode, percent_encode};

use super::Auth;

pub struct Encode<'a>(pub &'a Auth, pub bool);

impl Encode<'_> {
	pub fn domain(s: &str) -> PercentEncode<'_> {
		const SET: &AsciiSet = &CONTROLS.add(b'/').add(b':');
		percent_encode(s.as_bytes(), SET)
	}
}

impl fmt::Display for Encode<'_> {
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
