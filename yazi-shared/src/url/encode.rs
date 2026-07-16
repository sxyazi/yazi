use std::fmt::{self, Display};

use percent_encoding::{AsciiSet, CONTROLS, PercentEncode, percent_encode};

use crate::{auth::{EncodeAuth, EncodePrefix}, spec::EncodeSpec, url::Url};

#[derive(Clone, Copy)]
pub struct Encode<'a>(pub Url<'a>);

impl Encode<'_> {
	pub fn loc(b: &[u8]) -> PercentEncode<'_> {
		const SET: &AsciiSet = &CONTROLS.add(b'%');
		percent_encode(b, SET)
	}
}

impl Display for Encode<'_> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let loc = Self::loc(self.0.loc().encoded_bytes());
		match self.0 {
			Url::Regular(_) => write!(f, "regular~://{loc}"),
			Url::Search { auth, .. }
			| Url::Mount { auth, .. }
			| Url::Hub { auth, .. }
			| Url::Scope { auth, .. }
			| Url::Sftp { auth, .. } => {
				write!(
					f,
					"{}{}{}{loc}",
					EncodeAuth(auth, true),
					EncodeSpec::ports((*self).into()),
					EncodePrefix(auth)
				)
			}
		}
	}
}
