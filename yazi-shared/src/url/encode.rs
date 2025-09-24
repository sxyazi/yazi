use std::{fmt::{self, Display}, ops::Not};

use percent_encoding::{AsciiSet, CONTROLS, PercentEncode, percent_encode};

use crate::{loc::Loc, scheme::SchemeRef, url::{Url, UrlBuf}};

pub struct Encode<'a> {
	loc:    Loc<'a>,
	scheme: SchemeRef<'a>,
}

impl<'a> From<Url<'a>> for Encode<'a> {
	fn from(url: Url<'a>) -> Self { Self::new(url.loc, url.scheme) }
}

impl<'a> From<&'a UrlBuf> for Encode<'a> {
	fn from(url: &'a UrlBuf) -> Self { Self::new(url.loc.as_loc(), url.scheme.as_ref()) }
}

impl<'a> Encode<'a> {
	#[inline]
	pub(super) fn new(loc: Loc<'a>, scheme: SchemeRef<'a>) -> Self { Self { loc, scheme } }

	#[inline]
	pub fn domain<'s>(s: &'s str) -> PercentEncode<'s> {
		const SET: &AsciiSet = &CONTROLS.add(b'/').add(b':');
		percent_encode(s.as_bytes(), SET)
	}

	fn ports(&self) -> impl Display {
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
					SchemeRef::Regular => Ok(()),
					SchemeRef::Search(_) | SchemeRef::Archive(_) => w!(0, 0),
					SchemeRef::Sftp(_) => w!(
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
		use SchemeRef as S;
		match self.scheme {
			S::Regular => write!(f, "regular://"),
			S::Search(d) => write!(f, "search://{}{}/", Self::domain(d), self.ports()),
			S::Archive(d) => write!(f, "archive://{}{}/", Self::domain(d), self.ports()),
			S::Sftp(d) => write!(f, "sftp://{}{}/", Self::domain(d), self.ports()),
		}
	}
}

// --- Tilded
pub struct EncodeTilded<'a> {
	loc:    Loc<'a>,
	scheme: SchemeRef<'a>,
}

impl<'a> From<&'a UrlBuf> for EncodeTilded<'a> {
	fn from(url: &'a UrlBuf) -> Self { Self { loc: url.loc.as_loc(), scheme: url.scheme.as_ref() } }
}

impl<'a> From<&EncodeTilded<'a>> for Encode<'a> {
	fn from(value: &EncodeTilded<'a>) -> Self { Self::new(value.loc, value.scheme) }
}

impl Display for EncodeTilded<'_> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		use Encode as E;
		use SchemeRef as S;

		let loc = percent_encode(self.loc.as_os_str().as_encoded_bytes(), CONTROLS);
		match self.scheme {
			S::Regular => write!(f, "regular~://{loc}"),
			S::Search(d) => write!(f, "search~://{}{}/{loc}", E::domain(d), E::ports(&self.into())),
			S::Archive(d) => {
				write!(f, "archive~://{}{}/{loc}", E::domain(d), E::ports(&self.into()))
			}
			S::Sftp(d) => write!(f, "sftp~://{}{}/{loc}", E::domain(d), E::ports(&self.into())),
		}
	}
}
