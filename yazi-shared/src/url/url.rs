use std::{borrow::Cow, ffi::OsStr, fmt::{Debug, Formatter}, hash::BuildHasher, ops::Deref, path::{Path, PathBuf}, str::FromStr};

use anyhow::Result;
use percent_encoding::percent_decode;
use serde::{Deserialize, Serialize};

use super::UrnBuf;
use crate::{IntoOsStr, url::{Components, Display, Encode, EncodeTilded, Loc, Scheme, Urn}};

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

impl From<&Url> for Url {
	fn from(url: &Url) -> Self { url.clone() }
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
		let (scheme, path, port) = Self::parse(bytes)?;

		let loc =
			if let Some((uri, urn)) = port { Loc::with(path, uri, urn)? } else { Loc::from(path) };

		Ok(Self { loc, scheme })
	}
}

impl FromStr for Url {
	type Err = anyhow::Error;

	fn from_str(s: &str) -> Result<Self, Self::Err> { s.as_bytes().try_into() }
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
	pub fn base(&self) -> Url {
		use Scheme as S;

		let loc: Loc = self.loc.base().into();
		match self.scheme {
			S::Regular => Self { loc, scheme: S::Regular },
			S::Search(_) => Self { loc, scheme: self.scheme.clone() },
			S::Archive(_) => Self { loc, scheme: self.scheme.clone() },
			S::Sftp(_) => Self { loc, scheme: self.scheme.clone() },
		}
	}

	pub fn join(&self, path: impl AsRef<Path>) -> Self {
		use Scheme as S;

		let join = self.loc.join(path);

		let loc = match self.scheme {
			S::Regular => join.into(),
			S::Search(_) => Loc::new(join, self.loc.base(), self.loc.base()),
			S::Archive(_) => Loc::floated(join, self.loc.base()),
			S::Sftp(_) => join.into(),
		};

		Self { loc, scheme: self.scheme.clone() }
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

	pub fn strip_prefix(&self, base: impl AsRef<Url>) -> Option<&Urn> {
		use Scheme as S;

		let base = base.as_ref();
		let prefix = self.loc.strip_prefix(&base.loc).ok()?;

		Some(Urn::new(match (&self.scheme, &base.scheme) {
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
		}?))
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

	pub fn parse(bytes: &[u8]) -> Result<(Scheme, PathBuf, Option<(usize, usize)>)> {
		let mut skip = 0;
		let (scheme, tilde, port) = Scheme::parse(bytes, &mut skip)?;

		let rest = if tilde {
			Cow::from(percent_decode(&bytes[skip..])).into_os_str()?
		} else {
			bytes[skip..].into_os_str()?
		};

		let path = match rest {
			Cow::Borrowed(s) => Path::new(s).to_owned(),
			Cow::Owned(s) => PathBuf::from(s),
		};

		Ok((scheme, path, port))
	}
}

impl Url {
	// --- Regular
	#[inline]
	pub fn is_regular(&self) -> bool { self.scheme == Scheme::Regular }

	#[inline]
	pub fn to_regular(&self) -> Self {
		Self { loc: self.loc.to_path().into(), scheme: Scheme::Regular }
	}

	#[inline]
	pub fn into_regular(mut self) -> Self {
		self.loc = self.loc.into_path().into();
		self.scheme = Scheme::Regular;
		self
	}

	// --- Search
	#[inline]
	pub fn is_search(&self) -> bool { matches!(self.scheme, Scheme::Search(_)) }

	#[inline]
	pub fn to_search(&self, domain: impl AsRef<str>) -> Self {
		Self {
			loc:    Loc::zeroed(self.loc.to_path()),
			scheme: Scheme::Search(domain.as_ref().to_owned()),
		}
	}

	#[inline]
	pub fn into_search(mut self, domain: impl AsRef<str>) -> Self {
		self.loc = Loc::zeroed(self.loc.into_path());
		self.scheme = Scheme::Search(domain.as_ref().to_owned());
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
		write!(f, "{}{}", Encode::from(self), self.loc.display())
	}
}

impl Serialize for Url {
	fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
		let Url { scheme, loc } = self;
		match (scheme.is_virtual(), loc.to_str()) {
			(false, Some(s)) => serializer.serialize_str(s),
			(true, Some(s)) => serializer.serialize_str(&format!("{}{s}", Encode::from(self))),
			(_, None) => serializer.collect_str(&EncodeTilded::from(self)),
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

// --- Tests
#[cfg(test)]
mod tests {
	use anyhow::Result;

	use super::*;

	#[test]
	fn test_join() -> anyhow::Result<()> {
		let cases = [
			// Regular
			("/a", "b/c", "regular:///a/b/c"),
			// Search
			("search://kw//a", "b/c", "search://kw:2:2//a/b/c"),
			("search://kw:2:2//a/b/c", "d/e", "search://kw:4:4//a/b/c/d/e"),
			// Archive
			("archive:////a/b.zip", "c/d", "archive://:2:1//a/b.zip/c/d"),
			("archive://:2:1//a/b.zip/c/d", "e/f", "archive://:4:1//a/b.zip/c/d/e/f"),
			("archive://:2:2//a/b.zip/c/d", "e/f", "archive://:4:1//a/b.zip/c/d/e/f"),
			// SFTP
			("sftp://remote//a", "b/c", "sftp://remote:1:1//a/b/c"),
			("sftp://remote:1:1//a/b/c", "d/e", "sftp://remote:1:1//a/b/c/d/e"),
			// Relative
			("search://kw", "b/c", "search://kw:2:2/b/c"),
			("search://kw/", "b/c", "search://kw:2:2/b/c"),
		];

		for (base, path, expected) in cases {
			let base: Url = base.parse()?;
			#[cfg(unix)]
			assert_eq!(format!("{:?}", base.join(path)), expected);
			#[cfg(windows)]
			assert_eq!(format!("{:?}", base.join(path)).replace(r"\", "/"), expected.replace(r"\", "/"));
		}

		Ok(())
	}

	#[test]
	fn test_parent_url() -> anyhow::Result<()> {
		let cases = [
			// Regular
			("/a", Some("regular:///")),
			("/", None),
			// Search
			("search://kw:2:2//a/b/c", Some("search://kw:1:1//a/b")),
			("search://kw:1:1//a/b", Some("search://kw//a")),
			("search://kw//a", Some("regular:///")),
			// Archive
			("archive://:2:1//a/b.zip/c/d", Some("archive://:1:1//a/b.zip/c")),
			("archive://:1:1//a/b.zip/c", Some("archive:////a/b.zip")),
			("archive:////a/b.zip", Some("regular:///a")),
			// SFTP
			("sftp://remote:1:1//a/b", Some("sftp://remote:1:1//a")),
			("sftp://remote:1:1//a", Some("sftp://remote:1//")),
			("sftp://remote:1//", None),
			("sftp://remote//", None),
			// Relative
			("search://kw:2:2/a/b", Some("search://kw:1:1/a")),
			("search://kw:1:1/a", Some("search://kw/")),
			("search://kw/", None),
		];

		for (path, expected) in cases {
			let path: Url = path.parse()?;
			assert_eq!(path.parent_url().map(|u| format!("{:?}", u)).as_deref(), expected);
		}

		Ok(())
	}

	#[test]
	fn test_into_search() -> Result<()> {
		const S: char = std::path::MAIN_SEPARATOR;

		let u: Url = "/root".parse()?;
		assert_eq!(format!("{u:?}"), "regular:///root");

		let u = u.into_search("kw");
		assert_eq!(format!("{u:?}"), "search://kw//root");
		assert_eq!(format!("{:?}", u.parent_url().unwrap()), "regular:///");

		let u = u.join("examples");
		assert_eq!(format!("{u:?}"), format!("search://kw:1:1//root{S}examples"));

		let u = u.join("README.md");
		assert_eq!(format!("{u:?}"), format!("search://kw:2:2//root{S}examples{S}README.md"));

		let u = u.parent_url().unwrap();
		assert_eq!(format!("{u:?}"), format!("search://kw:1:1//root{S}examples"));

		let u = u.parent_url().unwrap();
		assert_eq!(format!("{u:?}"), "search://kw//root");

		let u = u.parent_url().unwrap();
		assert_eq!(format!("{u:?}"), "regular:///");

		Ok(())
	}
}
