use std::path::Path;

use crate::{loc::Loc, url::{Scheme, UrlBuf}};

pub struct Url<'a> {
	pub loc:    Loc<'a>,
	pub scheme: &'a Scheme,
}

impl<'a> From<&'a UrlBuf> for Url<'a> {
	fn from(value: &'a UrlBuf) -> Self { Self { loc: value.loc.as_loc(), scheme: &value.scheme } }
}

impl From<Url<'_>> for UrlBuf {
	fn from(value: Url<'_>) -> Self { Self { loc: value.loc.into(), scheme: value.scheme.clone() } }
}

impl<'a> Url<'a> {
	pub fn regular(path: &'a Path) -> Self { Self { loc: path.into(), scheme: &Scheme::Regular } }

	pub fn base(&self) -> Option<Self> {
		use Scheme as S;

		if !self.loc.has_base() {
			return None;
		}

		let loc: Loc = self.loc.base().into();
		Some(match self.scheme {
			S::Regular => Self { loc, scheme: &S::Regular },
			S::Search(_) => Self { loc, scheme: &self.scheme },
			S::Archive(_) => Self { loc, scheme: &self.scheme },
			S::Sftp(_) => Self { loc, scheme: &self.scheme },
		})
	}
}
