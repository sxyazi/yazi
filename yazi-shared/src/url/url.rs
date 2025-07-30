use std::{borrow::Cow, ffi::OsStr, fmt::{Debug, Formatter}, hash::BuildHasher, ops::Deref, path::{Path, PathBuf}};

use anyhow::Result;
use percent_encoding::percent_decode;
use serde::{Deserialize, Serialize};

use super::UrnBuf;
use crate::{IntoOsStr, url::{Components, Display, Loc, Scheme}};

#[derive(Clone, Default, Eq, Ord, PartialOrd, PartialEq, Hash)]
pub struct Url {
	pub loc:    Loc,
	pub scheme: Scheme,
}

impl Deref for Url {
	type Target = Loc;

	fn deref(&self) -> &Self::Target { &self.loc }
}

impl From<Loc> for Url {
	fn from(loc: Loc) -> Self { Self { loc, scheme: Scheme::Regular } }
}

impl From<PathBuf> for Url {
	fn from(path: PathBuf) -> Self { Loc::from(path).into() }
}

impl From<&PathBuf> for Url {
	fn from(path: &PathBuf) -> Self { path.to_owned().into() }
}

impl From<&Path> for Url {
	fn from(path: &Path) -> Self { path.to_path_buf().into() }
}

impl TryFrom<&[u8]> for Url {
	type Error = anyhow::Error;

	fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
		let (scheme, path) = Self::parse(bytes)?;
		Ok(Self { loc: path.into(), scheme })
	}
}

impl TryFrom<&str> for Url {
	type Error = anyhow::Error;

	fn try_from(value: &str) -> Result<Self, Self::Error> { value.as_bytes().try_into() }
}

impl TryFrom<String> for Url {
	type Error = anyhow::Error;

	fn try_from(value: String) -> Result<Self, Self::Error> { value.as_bytes().try_into() }
}

impl AsRef<Url> for Url {
	fn as_ref(&self) -> &Url { self }
}

// FIXME: remove
impl AsRef<Path> for Url {
	fn as_ref(&self) -> &Path { &self.loc }
}

impl<'a> From<&'a Url> for Cow<'a, Url> {
	fn from(url: &'a Url) -> Self { Cow::Borrowed(url) }
}

impl From<Url> for Cow<'_, Url> {
	fn from(url: Url) -> Self { Cow::Owned(url) }
}

impl From<Cow<'_, Url>> for Url {
	fn from(url: Cow<'_, Url>) -> Self { url.into_owned() }
}

impl Url {
	#[inline]
	pub fn with(&self, loc: impl Into<Loc>) -> Self {
		let loc: Loc = loc.into();
		// FIXME: simplify this
		Self { loc: Loc::with(self.loc.base(), loc.into_path()), scheme: self.scheme.clone() }
	}

	#[inline]
	pub fn base(&self) -> Url {
		let loc: Loc = self.loc.base().into();
		match &self.scheme {
			Scheme::Regular => Self { loc, scheme: Scheme::Regular },
			Scheme::Search(_) => self.with(loc),
			Scheme::SearchItem => Self { loc, scheme: Scheme::Search(String::new()) },
			Scheme::Archive(_) => self.with(loc),
			Scheme::Sftp(_) => self.with(loc),
		}
	}

	pub fn join(&self, path: impl AsRef<Path>) -> Self {
		match self.scheme {
			Scheme::Regular => Self { loc: self.loc.join(path).into(), scheme: Scheme::Regular },
			Scheme::Search(_) => {
				Self { loc: Loc::with(&self.loc, self.loc.join(path)), scheme: Scheme::SearchItem }
			}
			Scheme::SearchItem => {
				Self { loc: Loc::with(self.loc.base(), self.loc.join(path)), scheme: Scheme::SearchItem }
			}
			Scheme::Archive(_) => self.with(self.loc.join(path)),
			Scheme::Sftp(_) => self.with(self.loc.join(path)),
		}
	}

	#[inline]
	pub fn components(&self) -> Components<'_> { Components::new(self) }

	#[inline]
	pub fn covariant(&self, other: &Self) -> bool {
		self.scheme.covariant(&other.scheme) && self.loc == other.loc
	}

	#[inline]
	pub fn display(&self) -> Display<'_> { Display::new(self) }

	#[inline]
	pub fn os_str(&self) -> Cow<'_, OsStr> { self.components().os_str() }

	pub fn parent_url(&self) -> Option<Url> {
		let parent = self.loc.parent()?;
		let base = self.loc.base();

		Some(match &self.scheme {
			Scheme::Regular => Self { loc: parent.into(), scheme: Scheme::Regular },
			Scheme::Search(_) => Self { loc: parent.into(), scheme: Scheme::Regular },
			Scheme::SearchItem if parent == base => {
				Self { loc: parent.into(), scheme: Scheme::Search(String::new()) }
			}
			Scheme::SearchItem => {
				Self { loc: Loc::with(base, parent.to_owned()), scheme: Scheme::SearchItem }
			}
			Scheme::Archive(_) => self.with(parent),
			Scheme::Sftp(_) => self.with(parent),
		})
	}

	pub fn strip_prefix(&self, base: impl AsRef<Url>) -> Option<Self> {
		let base = base.as_ref();
		if !self.scheme.covariant(&base.scheme) {
			return None;
		}

		Some(self.with(self.loc.strip_prefix(&base.loc).ok()?))
	}

	#[inline]
	pub fn as_path(&self) -> Option<&Path> {
		Some(self.loc.as_path()).filter(|_| !self.scheme.is_virtual())
	}

	#[inline]
	pub fn set_name(&mut self, name: impl AsRef<OsStr>) { self.loc.set_name(name); }

	#[inline]
	pub fn pair(&self) -> Option<(Self, UrnBuf)> { Some((self.parent_url()?, self.loc.urn_owned())) }

	#[inline]
	pub fn hash_u64(&self) -> u64 { foldhash::fast::FixedState::default().hash_one(self) }

	#[inline]
	pub fn rebase(&self, parent: &Path) -> Self {
		debug_assert!(self.is_regular());
		self.loc.rebase(parent).into()
	}

	pub fn parse(bytes: &[u8]) -> Result<(Scheme, Cow<'_, Path>)> {
		let (scheme, skip, tilde) = Scheme::parse(bytes)?;

		let rest = &bytes[skip + tilde as usize..];
		let rest =
			if tilde { Cow::from(percent_decode(rest)).into_os_str()? } else { rest.into_os_str()? };

		Ok((scheme, match rest {
			Cow::Borrowed(s) => Path::new(s).into(),
			Cow::Owned(s) => PathBuf::from(s).into(),
		}))
	}
}

impl Url {
	// --- Regular
	#[inline]
	pub fn is_regular(&self) -> bool { self.scheme == Scheme::Regular }

	#[inline]
	pub fn to_regular(&self) -> Self { Self { loc: self.loc.clone(), scheme: Scheme::Regular } }

	#[inline]
	pub fn into_regular(mut self) -> Self {
		self.scheme = Scheme::Regular;
		self
	}

	// --- Search
	#[inline]
	pub fn is_search(&self) -> bool { matches!(self.scheme, Scheme::Search(_)) }

	#[inline]
	pub fn to_search(&self, frag: impl AsRef<str>) -> Self {
		Self { loc: self.loc.clone(), scheme: Scheme::Search(frag.as_ref().to_owned()) }
	}

	#[inline]
	pub fn into_search(mut self, frag: impl AsRef<str>) -> Self {
		self.scheme = Scheme::Search(frag.as_ref().to_owned());
		self
	}

	// --- Archive
	#[inline]
	pub fn is_archive(&self) -> bool { matches!(self.scheme, Scheme::Archive(_)) }

	// FIXME: remove
	#[inline]
	pub fn into_path(self) -> PathBuf { self.loc.into_path() }
}

impl Debug for Url {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}{}", self.scheme, self.loc.display())
	}
}

impl Serialize for Url {
	fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
		let Url { scheme, loc } = self;
		match (scheme.is_virtual(), loc.to_str()) {
			(false, Some(s)) => serializer.serialize_str(s),
			(true, Some(s)) => serializer.serialize_str(&format!("{scheme}{s}")),
			(_, None) => serializer.serialize_str(&scheme.encode_tilded(loc)),
		}
	}
}

impl<'de> Deserialize<'de> for Url {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		let s = String::deserialize(deserializer)?;
		Self::try_from(s).map_err(serde::de::Error::custom)
	}
}
