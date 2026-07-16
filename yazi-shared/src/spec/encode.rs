use std::fmt::{self, Display};

use crate::{auth::{AuthKind, EncodeAuth, EncodePrefix}, url::Url};

#[derive(Clone, Copy)]
pub struct EncodeSpec<'a>(pub Url<'a>);

impl<'a> From<crate::url::Encode<'a>> for EncodeSpec<'a> {
	fn from(value: crate::url::Encode<'a>) -> Self { Self(value.0) }
}

impl<'a> EncodeSpec<'a> {
	pub(crate) fn ports(self) -> impl Display {
		struct D<'a>(EncodeSpec<'a>);

		impl Display for D<'_> {
			fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
				macro_rules! w {
					($default_uri:expr, $default_urn:expr) => {{
						let (uri, urn) = self.0.0.spec().ports();
						match (uri != $default_uri, urn != $default_urn) {
							(true, true) => write!(f, ":{uri}:{urn}"),
							(true, false) => write!(f, ":{uri}"),
							(false, true) => write!(f, "::{urn}"),
							(false, false) => Ok(()),
						}
					}};
				}

				match self.0.0.kind() {
					AuthKind::Regular => Ok(()),
					AuthKind::Search | AuthKind::Mount => w!(0, 0),
					AuthKind::Hub | AuthKind::Scope | AuthKind::Sftp => {
						w!(self.0.0.loc().name().is_some() as usize, self.0.0.loc().name().is_some() as usize)
					}
				}
			}
		}

		D(self)
	}
}

impl Display for EncodeSpec<'_> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self.0 {
			Url::Regular(_) => write!(f, "regular://"),
			Url::Search { auth, .. }
			| Url::Mount { auth, .. }
			| Url::Hub { auth, .. }
			| Url::Scope { auth, .. }
			| Url::Sftp { auth, .. } => {
				write!(f, "{}{}{}", EncodeAuth(auth, false), self.ports(), EncodePrefix(auth))
			}
		}
	}
}
