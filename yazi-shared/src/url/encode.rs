use std::{fmt::{self, Display}, ops::Not};

use percent_encoding::{AsciiSet, CONTROLS, PercentEncode, percent_encode};

use crate::{scheme::SchemeRef, url::{AsUrl, Url, UrlBuf}};

#[derive(Clone, Copy)]
pub struct Encode<'a>(pub Url<'a>);

impl<'a> From<&'a UrlBuf> for Encode<'a> {
	fn from(value: &'a UrlBuf) -> Self { Self(value.as_url()) }
}

impl<'a> Encode<'a> {
	#[inline]
	pub fn domain<'s>(s: &'s str) -> PercentEncode<'s> {
		const SET: &AsciiSet = &CONTROLS.add(b'/').add(b':');
		percent_encode(s.as_bytes(), SET)
	}

	fn ports(self) -> impl Display {
		struct D<'a>(Encode<'a>);

		impl Display for D<'_> {
			fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
				macro_rules! w {
					($default_uri:expr, $default_urn:expr) => {{
						let uri = self.0.0.loc.uri().count();
						let urn = self.0.0.loc.urn().count();
						match (uri != $default_uri, urn != $default_urn) {
							(true, true) => write!(f, ":{uri}:{urn}"),
							(true, false) => write!(f, ":{uri}"),
							(false, true) => write!(f, "::{urn}"),
							(false, false) => Ok(()),
						}
					}};
				}

				match self.0.0.scheme {
					SchemeRef::Regular => Ok(()),
					SchemeRef::Search(_) | SchemeRef::Archive(_) => w!(0, 0),
					SchemeRef::Sftp(_) => w!(
						self.0.0.loc.as_os_str().is_empty().not() as usize,
						self.0.0.loc.file_name().is_some() as usize
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
		match self.0.scheme {
			S::Regular => write!(f, "regular://"),
			S::Search(d) => write!(f, "search://{}{}/", Self::domain(d), self.ports()),
			S::Archive(d) => write!(f, "archive://{}{}/", Self::domain(d), self.ports()),
			S::Sftp(d) => write!(f, "sftp://{}{}/", Self::domain(d), self.ports()),
		}
	}
}

// --- Tilded
#[derive(Clone, Copy)]
pub struct EncodeTilded<'a>(pub Url<'a>);

impl<'a> From<&'a UrlBuf> for EncodeTilded<'a> {
	fn from(value: &'a UrlBuf) -> Self { Self(value.as_url()) }
}

impl<'a> From<EncodeTilded<'a>> for Encode<'a> {
	fn from(value: EncodeTilded<'a>) -> Self { Self(value.0) }
}

impl Display for EncodeTilded<'_> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		use Encode as E;
		use SchemeRef as S;

		let loc = percent_encode(self.0.loc.as_os_str().as_encoded_bytes(), CONTROLS);
		match self.0.scheme {
			S::Regular => write!(f, "regular~://{loc}"),
			S::Search(d) => write!(f, "search~://{}{}/{loc}", E::domain(d), E::ports((*self).into())),
			S::Archive(d) => {
				write!(f, "archive~://{}{}/{loc}", E::domain(d), E::ports((*self).into()))
			}
			S::Sftp(d) => write!(f, "sftp~://{}{}/{loc}", E::domain(d), E::ports((*self).into())),
		}
	}
}
