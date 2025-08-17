use std::{ops::Deref, path::Path};

use hashbrown::Equivalent;

use crate::{loc::{Loc, LocBuf}, url::{Components, Scheme, UrlBuf, UrnBuf}};

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Url<'a> {
	pub loc:    Loc<'a>,
	pub scheme: Scheme,
}

impl<'a> Deref for Url<'a> {
	type Target = Loc<'a>;

	fn deref(&self) -> &Self::Target { &self.loc }
}

// TODO: remove
impl<'a> From<&'a Url<'a>> for Url<'a> {
	fn from(value: &'a Url) -> Self { Self { loc: value.loc.as_loc(), scheme: value.scheme.clone() } }
}

impl<'a> From<&'a UrlBuf> for Url<'a> {
	fn from(value: &'a UrlBuf) -> Self {
		Self { loc: value.loc.as_loc(), scheme: value.scheme.clone() }
	}
}

impl<'a> From<&'a mut UrlBuf> for Url<'a> {
	fn from(value: &'a mut UrlBuf) -> Self { Self::from(&*value) }
}

// --- Eq
impl PartialEq<UrlBuf> for Url<'_> {
	fn eq(&self, other: &UrlBuf) -> bool { *self == other.as_url() }
}

// --- Hash
impl Equivalent<UrlBuf> for Url<'_> {
	fn equivalent(&self, key: &UrlBuf) -> bool { self == key }
}

impl<'a> Url<'a> {
	#[inline]
	pub fn regular<T: AsRef<Path> + ?Sized>(path: &'a T) -> Self {
		Self { loc: path.as_ref().into(), scheme: Scheme::Regular }
	}

	#[inline]
	pub fn is_regular(&self) -> bool { self.scheme == Scheme::Regular }

	#[inline]
	pub fn as_url(&'a self) -> Url<'a> { Self::from(self) }

	#[inline]
	pub fn to_owned(&self) -> UrlBuf { self.into() }

	pub fn base(&self) -> Option<Self> {
		use Scheme as S;

		if !self.loc.has_base() {
			return None;
		}

		let loc: Loc = self.loc.base().into();
		Some(match self.scheme {
			S::Regular => Self { loc, scheme: S::Regular },
			S::Search(_) => Self { loc, scheme: self.scheme.clone() },
			S::Archive(_) => Self { loc, scheme: self.scheme.clone() },
			S::Sftp(_) => Self { loc, scheme: self.scheme.clone() },
		})
	}

	// TODO: use Url instead as the return type
	pub fn parent_url(&self) -> Option<UrlBuf> {
		use Scheme as S;

		let parent = self.loc.parent()?;
		let uri = self.loc.uri();

		Some(match self.scheme {
			// Regular
			S::Regular => UrlBuf { loc: parent.into(), scheme: S::Regular },

			// Search
			S::Search(_) if uri.is_empty() => UrlBuf { loc: parent.into(), scheme: S::Regular },
			S::Search(_) => UrlBuf {
				loc:    LocBuf::new(parent, self.loc.base(), self.loc.base()),
				scheme: self.scheme.clone(),
			},

			// Archive
			S::Archive(_) if uri.is_empty() => UrlBuf { loc: parent.into(), scheme: S::Regular },
			S::Archive(_) if uri.nth(1).is_none() => {
				UrlBuf { loc: LocBuf::zeroed(parent), scheme: self.scheme.clone() }
			}
			S::Archive(_) => {
				UrlBuf { loc: LocBuf::floated(parent, self.loc.base()), scheme: self.scheme.clone() }
			}

			// SFTP
			S::Sftp(_) => UrlBuf { loc: parent.into(), scheme: self.scheme.clone() },
		})
	}

	#[inline]
	pub fn components(&self) -> Components<'_> { Components::from(self) }

	#[inline]
	pub fn covariant(&self, other: impl Into<Self>) -> bool {
		let other = other.into();
		self.scheme.covariant(&other.scheme) && self.loc == other.loc
	}

	// TODO: use Urn instead as the return type
	#[inline]
	pub fn pair(&self) -> Option<(UrlBuf, UrnBuf)> {
		Some((self.parent_url()?, self.loc.urn_owned()))
	}

	#[inline]
	pub fn as_path(&self) -> Option<&Path> { Some(&*self.loc).filter(|_| !self.scheme.is_virtual()) }
}
