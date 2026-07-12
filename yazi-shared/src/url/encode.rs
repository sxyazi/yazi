use std::fmt::{self, Display};

use percent_encoding::{CONTROLS, percent_encode};

use crate::{auth::Encode as EncodeAuth, url::Url};

#[derive(Clone, Copy)]
pub struct Encode<'a>(pub Url<'a>);

impl Display for Encode<'_> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		use crate::spec::Encode as E;

		let loc = percent_encode(self.0.loc().encoded_bytes(), CONTROLS);
		match self.0 {
			Url::Regular(_) => write!(f, "regular~://{loc}"),
			Url::Search { auth, .. }
			| Url::Mount { auth, .. }
			| Url::Scope { auth, .. }
			| Url::Sftp { auth, .. } => {
				write!(f, "{}{}/{loc}", EncodeAuth(auth, true), E::ports((*self).into()))
			}
		}
	}
}
