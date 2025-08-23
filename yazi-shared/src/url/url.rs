use std::{borrow::Cow, ffi::OsStr, fmt::{Debug, Formatter}, ops::Deref, path::Path};

use hashbrown::Equivalent;

use crate::{loc::{Loc, LocBuf}, url::{Components, Encode, Scheme, UrlBuf, UrnBuf}};

#[derive(Clone, Eq, Hash, PartialEq)]
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

// --- Debug
impl Debug for Url<'_> {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		if self.scheme == Scheme::Regular {
			write!(f, "{}", self.loc.display())
		} else {
			write!(f, "{}{}", Encode::from(self), self.loc.display())
		}
	}
}

impl<'a> Url<'a> {
	#[inline]
	pub fn regular<T: AsRef<Path> + ?Sized>(path: &'a T) -> Self {
		Self { loc: path.as_ref().into(), scheme: Scheme::Regular }
	}

	#[inline]
	pub fn is_regular(&self) -> bool { self.scheme == Scheme::Regular }

	#[inline]
	pub fn into_regular(self) -> Self {
		Self { loc: self.loc.as_path().into(), scheme: Scheme::Regular }
	}

	#[inline]
	pub fn is_search(&self) -> bool { matches!(self.scheme, Scheme::Search(_)) }

	#[inline]
	pub fn as_url(&'a self) -> Url<'a> { Self::from(self) }

	#[inline]
	pub fn to_owned(&self) -> UrlBuf { self.into() }

	pub fn join(&self, path: impl AsRef<Path>) -> UrlBuf {
		use Scheme as S;

		let join = self.loc.join(path);

		let loc = match self.scheme {
			S::Regular => join.into(),
			S::Search(_) => LocBuf::new(join, self.loc.base(), self.loc.base()),
			S::Archive(_) => LocBuf::floated(join, self.loc.base()),
			S::Sftp(_) => join.into(),
		};

		UrlBuf { loc, scheme: self.scheme.clone() }
	}

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

	pub fn parent_url(&self) -> Option<Self> {
		use Scheme as S;

		let parent = self.loc.parent()?;
		let uri = self.loc.uri();

		Some(match self.scheme {
			// Regular
			S::Regular => Self { loc: parent.into(), scheme: S::Regular },

			// Search
			S::Search(_) if uri.is_empty() => Self { loc: parent.into(), scheme: S::Regular },
			S::Search(_) => Self {
				loc:    Loc::new(parent, self.loc.base(), self.loc.base()),
				scheme: self.scheme.clone(),
			},

			// Archive
			S::Archive(_) if uri.is_empty() => Self { loc: parent.into(), scheme: S::Regular },
			S::Archive(_) if uri.nth(1).is_none() => {
				Self { loc: Loc::zeroed(parent), scheme: self.scheme.clone() }
			}
			S::Archive(_) => {
				Self { loc: Loc::floated(parent, self.loc.base()), scheme: self.scheme.clone() }
			}

			// SFTP
			S::Sftp(_) => Self { loc: parent.into(), scheme: self.scheme.clone() },
		})
	}

	#[inline]
	pub fn starts_with<'b>(&self, base: impl Into<Url<'b>>) -> bool {
		let base: Url = base.into();
		self.scheme.covariant(&base.scheme) && self.loc.starts_with(base.loc)
	}

	#[inline]
	pub fn ends_with<'b>(&self, child: impl Into<Url<'b>>) -> bool {
		let child: Url = child.into();
		self.scheme.covariant(&child.scheme) && self.loc.ends_with(child.loc)
	}

	#[inline]
	pub fn components(&self) -> Components<'_> { Components::from(self) }

	#[inline]
	pub fn os_str(&self) -> Cow<'_, OsStr> { self.components().os_str() }

	#[inline]
	pub fn covariant(&self, other: impl Into<Self>) -> bool {
		let other = other.into();
		self.scheme.covariant(&other.scheme) && self.loc == other.loc
	}

	#[inline]
	pub fn pair(&self) -> Option<(Url<'a>, UrnBuf)> {
		Some((self.parent_url()?, self.loc.urn_owned()))
	}

	#[inline]
	pub fn as_path(&self) -> Option<&Path> { Some(&*self.loc).filter(|_| !self.scheme.is_virtual()) }
}
