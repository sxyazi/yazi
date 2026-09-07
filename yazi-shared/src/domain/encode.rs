use std::{fmt::{self, Display}, ops::Deref};

use percent_encoding::{AsciiSet, CONTROLS, percent_encode};

use super::Domain;

impl Domain<'_> {
	pub(crate) fn encode(&self) -> Encode<'_> {
		const SET: &AsciiSet = &CONTROLS.add(b'/').add(b':').add(b'%');
		Encode(self, SET)
	}

	pub(crate) fn encode_parent(&self) -> Encode<'_> {
		const SET: &AsciiSet = &CONTROLS.add(b'/').add(b',').add(b'@').add(b'%');
		Encode(self, SET)
	}
}

// --- Encode
pub struct Encode<'a>(&'a [u8], &'static AsciiSet);

impl Deref for Encode<'_> {
	type Target = [u8];

	fn deref(&self) -> &Self::Target { self.0 }
}

impl Display for Encode<'_> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		for chunk in self.utf8_chunks() {
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
