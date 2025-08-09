use std::fmt::{self, Display};

use percent_encoding::{AsciiSet, CONTROLS, PercentEncode, percent_encode};

use crate::url::{Loc, Scheme, Url};

pub struct Encode<'a> {
	loc:    &'a Loc,
	scheme: &'a Scheme,
}

impl<'a> From<&'a Url> for Encode<'a> {
	fn from(url: &'a Url) -> Self { Self::new(&url.loc, &url.scheme) }
}

impl<'a> Encode<'a> {
	#[inline]
	pub(super) fn new(loc: &'a Loc, scheme: &'a Scheme) -> Self { Self { loc, scheme } }

	#[inline]
	fn domain<'s>(s: &'s str) -> PercentEncode<'s> {
		const SET: &AsciiSet = &CONTROLS.add(b'/').add(b':');
		percent_encode(s.as_bytes(), SET)
	}

	#[inline]
	fn urn(loc: &'a Loc) -> impl Display {
		struct D(usize, usize);

		impl Display for D {
			fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
				let (uri, urn) = (self.0, self.1);
				match (uri != 0, urn != 0) {
					(true, true) => write!(f, ":{uri}:{urn}"),
					(true, false) => write!(f, ":{uri}"),
					(false, true) => write!(f, "::{urn}"),
					(false, false) => Ok(()),
				}
			}
		}

		D(loc.uri().count(), loc.urn().count())
	}
}

impl Display for Encode<'_> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self.scheme {
			Scheme::Regular => write!(f, "regular://"),
			Scheme::Search(d) => write!(f, "search://{}{}/", Self::domain(d), Self::urn(self.loc)),
			Scheme::Archive(d) => write!(f, "archive://{}{}/", Self::domain(d), Self::urn(self.loc)),
			Scheme::Sftp(d) => write!(f, "sftp://{}{}/", Self::domain(d), Self::urn(self.loc)),
		}
	}
}

// --- Tilded
pub struct EncodeTilded<'a> {
	loc:    &'a Loc,
	scheme: &'a Scheme,
}

impl<'a> From<&'a Url> for EncodeTilded<'a> {
	fn from(url: &'a Url) -> Self { Self { loc: &url.loc, scheme: &url.scheme } }
}

impl Display for EncodeTilded<'_> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		use Encode as E;

		let loc = percent_encode(self.loc.as_os_str().as_encoded_bytes(), CONTROLS);
		match self.scheme {
			Scheme::Regular => write!(f, "regular~://{loc}"),
			Scheme::Search(d) => write!(f, "search~://{}{}/{loc}", E::domain(d), E::urn(self.loc)),
			Scheme::Archive(d) => write!(f, "archive~://{}{}/{loc}", E::domain(d), E::urn(self.loc)),
			Scheme::Sftp(d) => write!(f, "sftp~://{}{}/{loc}", E::domain(d), E::urn(self.loc)),
		}
	}
}
