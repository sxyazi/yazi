use std::fmt::{self, Display};

use percent_encoding::{AsciiSet, CONTROLS, PercentEncode, percent_encode};

use crate::{scheme::SchemeKind, url::Url};

#[derive(Clone, Copy)]
pub struct Encode<'a>(pub Url<'a>);

impl<'a> From<crate::url::Encode<'a>> for Encode<'a> {
	fn from(value: crate::url::Encode<'a>) -> Self { Self(value.0) }
}

impl<'a> Encode<'a> {
	#[inline]
	pub fn domain<'s>(s: &'s str) -> PercentEncode<'s> {
		const SET: &AsciiSet = &CONTROLS.add(b'/').add(b':');
		percent_encode(s.as_bytes(), SET)
	}

	pub(crate) fn ports(self) -> impl Display {
		struct D<'a>(Encode<'a>);

		impl Display for D<'_> {
			fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
				macro_rules! w {
					($default_uri:expr, $default_urn:expr) => {{
						let (uri, urn) = self.0.0.scheme().ports();
						match (uri != $default_uri, urn != $default_urn) {
							(true, true) => write!(f, ":{uri}:{urn}"),
							(true, false) => write!(f, ":{uri}"),
							(false, true) => write!(f, "::{urn}"),
							(false, false) => Ok(()),
						}
					}};
				}

				match self.0.0.kind() {
					SchemeKind::Regular => Ok(()),
					SchemeKind::Search | SchemeKind::Archive => w!(0, 0),
					SchemeKind::Sftp => {
						w!(self.0.0.loc().name().is_some() as usize, self.0.0.loc().name().is_some() as usize)
					}
				}
			}
		}

		D(self)
	}
}

impl Display for Encode<'_> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self.0 {
			Url::Regular(_) => write!(f, "regular://"),
			Url::Search { domain, .. } => write!(f, "search://{}{}/", Self::domain(domain), self.ports()),
			Url::Archive { domain, .. } => {
				write!(f, "archive://{}{}/", Self::domain(domain), self.ports())
			}
			Url::Sftp { domain, .. } => write!(f, "sftp://{}{}/", Self::domain(domain), self.ports()),
		}
	}
}
