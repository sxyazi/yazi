use std::fmt::{self, Display};

use percent_encoding::{CONTROLS, percent_encode};

use crate::url::Url;

#[derive(Clone, Copy)]
pub struct Encode<'a>(pub Url<'a>);

impl Display for Encode<'_> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		use crate::scheme::Encode as E;

		let loc = percent_encode(self.0.loc().encoded_bytes(), CONTROLS);
		match self.0 {
			Url::Regular(_) => write!(f, "regular~://{loc}"),
			Url::Search { domain, .. } => {
				write!(f, "search~://{}{}/{loc}", E::domain(domain), E::ports((*self).into()))
			}
			Url::Archive { domain, .. } => {
				write!(f, "archive~://{}{}/{loc}", E::domain(domain), E::ports((*self).into()))
			}
			Url::Sftp { domain, .. } => {
				write!(f, "sftp~://{}{}/{loc}", E::domain(domain), E::ports((*self).into()))
			}
		}
	}
}
