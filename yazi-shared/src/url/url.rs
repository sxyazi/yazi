use std::{borrow::Cow, ffi::OsStr, fmt::{Debug, Formatter}, path::{Path, PathBuf}};

use hashbrown::Equivalent;

use crate::{loc::{Loc, LocBuf}, path::{PathDyn, PathLike}, scheme::SchemeRef, url::{AsUrl, Components, Encode, UrlBuf}};

#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub struct Url<'a> {
	pub loc:    Loc<'a>,
	pub scheme: SchemeRef<'a>,
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
		if self.scheme == SchemeRef::Regular {
			write!(f, "{}", self.loc.display())
		} else {
			write!(f, "{}{}", Encode(*self), self.loc.display())
		}
	}
}

impl<'a> Url<'a> {
	#[inline]
	pub fn regular<T: AsRef<Path> + ?Sized>(path: &'a T) -> Self {
		Self { loc: Loc::bare(path.as_ref()), scheme: SchemeRef::Regular }
	}

	#[inline]
	pub fn is_regular(self) -> bool { self.scheme == SchemeRef::Regular }

	#[inline]
	pub fn into_regular(self) -> Self {
		Self { loc: Loc::bare(self.loc.as_path()), scheme: SchemeRef::Regular }
	}

	#[inline]
	pub fn is_search(self) -> bool { matches!(self.scheme, SchemeRef::Search(_)) }

	#[inline]
	pub fn is_absolute(self) -> bool {
		use SchemeRef as S;

		match self.scheme {
			S::Regular | S::Search(_) => self.loc.is_absolute(),
			S::Archive(_) | S::Sftp(_) => self.loc.has_root(),
		}
	}

	#[inline]
	pub fn has_root(self) -> bool { self.loc.has_root() }

	#[inline]
	pub fn to_owned(self) -> UrlBuf { self.into() }

	pub fn join(self, path: impl AsRef<Path>) -> UrlBuf {
		use SchemeRef as S;

		let join = self.loc.join(path);

		let loc = match self.scheme {
			S::Regular => join.into(),
			S::Search(_) => LocBuf::<PathBuf>::new(join, self.loc.base(), self.loc.base()),
			S::Archive(_) => LocBuf::<PathBuf>::floated(join, self.loc.base()),
			S::Sftp(_) => join.into(),
		};

		UrlBuf { loc, scheme: self.scheme.into() }
	}

	pub fn strip_prefix(self, base: impl AsUrl) -> Option<&'a Path> {
		use SchemeRef as S;

		let base = base.as_url();
		let prefix = self.loc.strip_prefix(base.loc)?;

		Some(match (self.scheme, base.scheme) {
			// Same scheme
			(S::Regular, S::Regular) => Some(prefix),
			(S::Search(_), S::Search(_)) => Some(prefix),
			(S::Archive(a), S::Archive(b)) => Some(prefix).filter(|_| a == b),
			(S::Sftp(a), S::Sftp(b)) => Some(prefix).filter(|_| a == b),

			// Both are local files
			(S::Regular, S::Search(_)) => Some(prefix),
			(S::Search(_), S::Regular) => Some(prefix),

			// Only the entry of archives is a local file
			(S::Regular, S::Archive(_)) => Some(prefix).filter(|_| base.uri().is_empty()),
			(S::Search(_), S::Archive(_)) => Some(prefix).filter(|_| base.uri().is_empty()),
			(S::Archive(_), S::Regular) => Some(prefix).filter(|_| self.uri().is_empty()),
			(S::Archive(_), S::Search(_)) => Some(prefix).filter(|_| self.uri().is_empty()),

			// Independent virtual file space
			(S::Regular, S::Sftp(_)) => None,
			(S::Search(_), S::Sftp(_)) => None,
			(S::Archive(_), S::Sftp(_)) => None,
			(S::Sftp(_), S::Regular) => None,
			(S::Sftp(_), S::Search(_)) => None,
			(S::Sftp(_), S::Archive(_)) => None,
		}?)
	}

	#[inline]
	pub fn uri(self) -> PathDyn<'a> { self.loc.uri().into() }

	#[inline]
	pub fn urn(self) -> &'a Path { self.loc.urn() }

	#[inline]
	pub fn name(self) -> Option<&'a OsStr> { self.loc.name() }

	#[inline]
	pub fn stem(self) -> Option<&'a OsStr> { self.loc.stem() }

	#[inline]
	pub fn ext(self) -> Option<&'a OsStr> { self.loc.ext() }

	pub fn base(self) -> Option<Self> {
		use SchemeRef as S;

		if !self.loc.has_base() {
			return None;
		}

		let loc = Loc::bare(self.loc.base());
		Some(match self.scheme {
			S::Regular => Self { loc, scheme: S::Regular },
			S::Search(_) => Self { loc, scheme: self.scheme },
			S::Archive(_) => Self { loc, scheme: self.scheme },
			S::Sftp(_) => Self { loc, scheme: self.scheme },
		})
	}

	pub fn parent(self) -> Option<Self> {
		use SchemeRef as S;

		let parent = self.loc.parent()?;
		let uri = self.loc.uri();

		Some(match self.scheme {
			// Regular
			S::Regular => Self { loc: Loc::bare(parent), scheme: S::Regular },

			// Search
			S::Search(_) if uri.as_os_str().is_empty() => {
				Self { loc: Loc::bare(parent), scheme: S::Regular }
			}
			S::Search(_) => {
				Self { loc: Loc::new(parent, self.loc.base(), self.loc.base()), scheme: self.scheme }
			}

			// Archive
			S::Archive(_) if uri.as_os_str().is_empty() => {
				Self { loc: Loc::bare(parent), scheme: S::Regular }
			}
			S::Archive(_) if uri.components().nth(1).is_none() => {
				Self { loc: Loc::zeroed(parent), scheme: self.scheme }
			}
			S::Archive(_) => Self { loc: Loc::floated(parent, self.loc.base()), scheme: self.scheme },

			// SFTP
			S::Sftp(_) => Self { loc: Loc::bare(parent), scheme: self.scheme },
		})
	}

	#[inline]
	pub fn starts_with(self, base: impl AsUrl) -> bool {
		let base = base.as_url();
		self.scheme.covariant(base.scheme) && self.loc.starts_with(base.loc)
	}

	#[inline]
	pub fn ends_with(self, child: impl AsUrl) -> bool {
		let child = child.as_url();
		self.scheme.covariant(child.scheme) && self.loc.ends_with(child.loc)
	}

	#[inline]
	pub fn components(self) -> Components<'a> { Components::from(self) }

	#[inline]
	pub fn os_str(self) -> Cow<'a, OsStr> { self.components().os_str() }

	#[inline]
	pub fn covariant(self, other: impl AsUrl) -> bool {
		let other = other.as_url();
		self.scheme.covariant(other.scheme) && self.loc == other.loc
	}

	#[inline]
	pub fn pair(self) -> Option<(Self, &'a Path)> { Some((self.parent()?, self.loc.urn())) }

	#[inline]
	pub fn as_path(self) -> Option<&'a Path> {
		Some(self.loc.as_path()).filter(|_| self.scheme.is_local())
	}

	#[inline]
	pub fn has_base(self) -> bool { self.loc.has_base() }

	#[inline]
	pub fn has_trail(self) -> bool { self.loc.has_trail() }
}
