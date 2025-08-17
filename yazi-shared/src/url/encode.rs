use std::{fmt::{self, Display}, ops::Not};

use percent_encoding::{AsciiSet, CONTROLS, PercentEncode, percent_encode};

use crate::{loc::Loc, url::{Scheme, Url, UrlBuf}};

pub struct Encode<'a> {
	loc:    Loc<'a>,
	scheme: &'a Scheme,
}

impl<'a> From<&'a Url<'a>> for Encode<'a> {
	fn from(url: &'a Url<'a>) -> Self { Self::new(url.loc, &url.scheme) }
}

impl<'a> From<&'a UrlBuf> for Encode<'a> {
	fn from(url: &'a UrlBuf) -> Self { Self::new(url.loc.as_loc(), &url.scheme) }
}

impl<'a> Encode<'a> {
	#[inline]
	pub(super) fn new(loc: Loc<'a>, scheme: &'a Scheme) -> Self { Self { loc, scheme } }

	#[inline]
	fn domain<'s>(s: &'s str) -> PercentEncode<'s> {
		const SET: &AsciiSet = &CONTROLS.add(b'/').add(b':');
		percent_encode(s.as_bytes(), SET)
	}

	fn urn(&self) -> impl Display {
		struct D<'a>(&'a Encode<'a>);

		impl Display for D<'_> {
			fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
				macro_rules! w {
					($default_uri:expr, $default_urn:expr) => {{
						let uri = self.0.loc.uri().count();
						let urn = self.0.loc.urn().count();
						match (uri != $default_uri, urn != $default_urn) {
							(true, true) => write!(f, ":{uri}:{urn}"),
							(true, false) => write!(f, ":{uri}"),
							(false, true) => write!(f, "::{urn}"),
							(false, false) => Ok(()),
						}
					}};
				}

				match self.0.scheme {
					Scheme::Regular => Ok(()),
					Scheme::Search(_) | Scheme::Archive(_) => w!(0, 0),
					Scheme::Sftp(_) => w!(
						self.0.loc.as_os_str().is_empty().not() as usize,
						self.0.loc.file_name().is_some() as usize
					),
				}
			}
		}

		D(self)
	}
}

impl Display for Encode<'_> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self.scheme {
			Scheme::Regular => write!(f, "regular://"),
			Scheme::Search(d) => write!(f, "search://{}{}/", Self::domain(d), self.urn()),
			Scheme::Archive(d) => write!(f, "archive://{}{}/", Self::domain(d), self.urn()),
			Scheme::Sftp(d) => write!(f, "sftp://{}{}/", Self::domain(d), self.urn()),
		}
	}
}

// --- Tilded
pub struct EncodeTilded<'a> {
	loc:    Loc<'a>,
	scheme: &'a Scheme,
}

impl<'a> From<&'a Url<'a>> for EncodeTilded<'a> {
	fn from(url: &'a Url<'a>) -> Self { Self { loc: url.loc, scheme: &url.scheme } }
}

impl<'a> From<&'a UrlBuf> for EncodeTilded<'a> {
	fn from(url: &'a UrlBuf) -> Self { Self { loc: url.loc.as_loc(), scheme: &url.scheme } }
}

impl<'a> From<&'a EncodeTilded<'a>> for Encode<'a> {
	fn from(value: &'a EncodeTilded<'a>) -> Self { Self::new(value.loc, value.scheme) }
}

impl Display for EncodeTilded<'_> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		use Encode as E;

		let loc = percent_encode(self.loc.as_os_str().as_encoded_bytes(), CONTROLS);
		match self.scheme {
			Scheme::Regular => write!(f, "regular~://{loc}"),
			Scheme::Search(d) => write!(f, "search~://{}{}/{loc}", E::domain(d), E::urn(&self.into())),
			Scheme::Archive(d) => write!(f, "archive~://{}{}/{loc}", E::domain(d), E::urn(&self.into())),
			Scheme::Sftp(d) => write!(f, "sftp~://{}{}/{loc}", E::domain(d), E::urn(&self.into())),
		}
	}
}
