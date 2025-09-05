use std::{borrow::Cow, ffi::OsStr, fmt::{Debug, Formatter}, hash::BuildHasher, path::{Path, PathBuf}, str::FromStr};

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::{loc::LocBuf, pool::Pool, scheme::{Scheme, SchemeRef}, url::{Components, Display, Encode, EncodeTilded, Uri, Url, UrlCow, Urn}};

#[derive(Clone, Default, Eq, Hash, PartialEq)]
pub struct UrlBuf {
	pub loc:    LocBuf,
	pub scheme: Scheme,
}

impl From<LocBuf> for UrlBuf {
	fn from(loc: LocBuf) -> Self { Self { loc, scheme: Scheme::Regular } }
}

impl From<PathBuf> for UrlBuf {
	fn from(path: PathBuf) -> Self { LocBuf::from(path).into() }
}

impl From<&Url<'_>> for UrlBuf {
	fn from(url: &Url<'_>) -> Self { Self { loc: url.loc.into(), scheme: url.scheme.into() } }
}

impl From<Url<'_>> for UrlBuf {
	fn from(url: Url<'_>) -> Self { Self { loc: url.loc.into(), scheme: url.scheme.into() } }
}

impl From<&Self> for UrlBuf {
	fn from(url: &Self) -> Self { url.clone() }
}

impl From<&PathBuf> for UrlBuf {
	fn from(path: &PathBuf) -> Self { path.to_owned().into() }
}

impl From<&Path> for UrlBuf {
	fn from(path: &Path) -> Self { path.to_path_buf().into() }
}

impl FromStr for UrlBuf {
	type Err = anyhow::Error;

	fn from_str(s: &str) -> Result<Self, Self::Err> { Ok(UrlCow::try_from(s)?.into_owned()) }
}

impl TryFrom<String> for UrlBuf {
	type Error = anyhow::Error;

	fn try_from(value: String) -> Result<Self, Self::Error> {
		Ok(UrlCow::try_from(value)?.into_owned())
	}
}

impl AsRef<Self> for UrlBuf {
	fn as_ref(&self) -> &Self { self }
}

impl<'a> From<&'a UrlBuf> for Cow<'a, UrlBuf> {
	fn from(url: &'a UrlBuf) -> Self { Cow::Borrowed(url) }
}

impl From<UrlBuf> for Cow<'_, UrlBuf> {
	fn from(url: UrlBuf) -> Self { Cow::Owned(url) }
}

impl From<Cow<'_, Self>> for UrlBuf {
	fn from(url: Cow<'_, Self>) -> Self { url.into_owned() }
}

// --- Eq
impl PartialEq<Url<'_>> for UrlBuf {
	fn eq(&self, other: &Url) -> bool { self.as_url() == *other }
}

impl PartialEq<Url<'_>> for &UrlBuf {
	fn eq(&self, other: &Url) -> bool { self.as_url() == *other }
}

impl UrlBuf {
	#[inline]
	pub fn join(&self, path: impl AsRef<Path>) -> Self { self.as_url().join(path) }

	#[inline]
	pub fn components(&self) -> Components<'_> { Components::from(self) }

	#[inline]
	pub fn os_str(&self) -> Cow<'_, OsStr> { self.components().os_str() }

	#[inline]
	pub fn display(&self) -> Display<'_> { Display::new(self) }

	#[inline]
	pub fn covariant(&self, other: &Self) -> bool { self.as_url().covariant(other) }

	#[inline]
	pub fn parent(&self) -> Option<Url<'_>> { self.as_url().parent() }

	#[inline]
	pub fn starts_with<'a>(&self, base: impl Into<Url<'a>>) -> bool {
		self.as_url().starts_with(base)
	}

	#[inline]
	pub fn ends_with<'a>(&self, child: impl Into<Url<'a>>) -> bool { self.as_url().ends_with(child) }

	pub fn strip_prefix<'a>(&self, base: impl Into<Url<'a>>) -> Option<&Urn> {
		use Scheme as S;
		use SchemeRef as T;

		let base = base.into();
		let prefix = self.loc.strip_prefix(base.loc).ok()?;

		Some(Urn::new(match (&self.scheme, base.scheme) {
			// Same scheme
			(S::Regular, T::Regular) => Some(prefix),
			(S::Search(_), T::Search(_)) => Some(prefix),
			(S::Archive(a), T::Archive(b)) => Some(prefix).filter(|_| a == b),
			(S::Sftp(a), T::Sftp(b)) => Some(prefix).filter(|_| a == b),

			// Both are local files
			(S::Regular, T::Search(_)) => Some(prefix),
			(S::Search(_), T::Regular) => Some(prefix),

			// Only the entry of archives is a local file
			(S::Regular, T::Archive(_)) => Some(prefix).filter(|_| base.uri().is_empty()),
			(S::Search(_), T::Archive(_)) => Some(prefix).filter(|_| base.uri().is_empty()),
			(S::Archive(_), T::Regular) => Some(prefix).filter(|_| self.uri().is_empty()),
			(S::Archive(_), T::Search(_)) => Some(prefix).filter(|_| self.uri().is_empty()),

			// Independent virtual file space
			(S::Regular, T::Sftp(_)) => None,
			(S::Search(_), T::Sftp(_)) => None,
			(S::Archive(_), T::Sftp(_)) => None,
			(S::Sftp(_), T::Regular) => None,
			(S::Sftp(_), T::Search(_)) => None,
			(S::Sftp(_), T::Archive(_)) => None,
		}?))
	}

	#[inline]
	pub fn as_path(&self) -> Option<&Path> { self.as_url().as_path() }

	#[inline]
	pub fn into_path(self) -> Option<PathBuf> {
		Some(self.loc.into_path()).filter(|_| !self.scheme.is_virtual())
	}

	#[inline]
	pub fn set_name(&mut self, name: impl AsRef<OsStr>) { self.loc.set_name(name); }

	#[inline]
	pub fn rebase(&self, base: &Path) -> Self {
		Self { loc: self.loc.rebase(base), scheme: self.scheme.clone() }
	}

	#[inline]
	pub fn pair(&self) -> Option<(Url<'_>, &Urn)> { self.as_url().pair() }

	#[inline]
	pub fn hash_u64(&self) -> u64 { foldhash::fast::FixedState::default().hash_one(self) }
}

impl UrlBuf {
	#[inline]
	pub fn as_url(&self) -> Url<'_> { Url::from(self) }

	#[inline]
	pub fn base(&self) -> Option<Url<'_>> { self.as_url().base() }
}

impl UrlBuf {
	// --- Regular
	#[inline]
	pub fn is_regular(&self) -> bool { self.as_url().is_regular() }

	#[inline]
	pub fn to_regular(&self) -> Self { self.as_url().into_regular().into() }

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
			loc:    LocBuf::zeroed(self.loc.to_path()),
			scheme: Scheme::Search(Pool::<str>::intern(domain)),
		}
	}

	#[inline]
	pub fn into_search(mut self, domain: impl AsRef<str>) -> Self {
		self.loc = LocBuf::zeroed(self.loc.into_path());
		self.scheme = Scheme::Search(Pool::<str>::intern(domain));
		self
	}

	// --- Archive
	#[inline]
	pub fn is_archive(&self) -> bool { matches!(self.scheme, Scheme::Archive(_)) }

	// --- Internal
	#[inline]
	pub fn is_internal(&self) -> bool {
		match self.scheme {
			Scheme::Regular | Scheme::Sftp(_) => true,
			Scheme::Search(_) => !self.loc.uri().is_empty(),
			Scheme::Archive(_) => false,
		}
	}

	// FIXME: remove
	#[inline]
	pub fn into_path2(self) -> PathBuf { self.loc.into_path() }

	#[inline]
	pub fn name(&self) -> Option<&OsStr> { self.as_url().name() }

	#[inline]
	pub fn stem(&self) -> Option<&OsStr> { self.as_url().stem() }

	#[inline]
	pub fn ext(&self) -> Option<&OsStr> { self.as_url().ext() }

	#[inline]
	pub fn uri(&self) -> &Uri { self.as_url().uri() }

	#[inline]
	pub fn urn(&self) -> &Urn { self.as_url().urn() }

	#[inline]
	pub fn is_absolute(&self) -> bool { self.as_url().is_absolute() }

	#[inline]
	pub fn has_root(&self) -> bool { self.as_url().has_root() }

	#[inline]
	pub fn has_trail(&self) -> bool { self.as_url().has_trail() }
}

impl Debug for UrlBuf {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result { self.as_url().fmt(f) }
}

impl Serialize for UrlBuf {
	fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
		let Self { scheme, loc } = self;
		match (scheme.is_virtual(), loc.to_str()) {
			(false, Some(s)) => serializer.serialize_str(s),
			(true, Some(s)) => serializer.serialize_str(&format!("{}{s}", Encode::from(self))),
			(_, None) => serializer.collect_str(&EncodeTilded::from(self)),
		}
	}
}

impl<'de> Deserialize<'de> for UrlBuf {
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
		crate::init_tests();
		let cases = [
			// Regular
			("/a", "b/c", "/a/b/c"),
			// Search
			("search://kw//a", "b/c", "search://kw:2:2//a/b/c"),
			("search://kw:2:2//a/b/c", "d/e", "search://kw:4:4//a/b/c/d/e"),
			// Archive
			("archive:////a/b.zip", "c/d", "archive://:2:1//a/b.zip/c/d"),
			("archive://:2:1//a/b.zip/c/d", "e/f", "archive://:4:1//a/b.zip/c/d/e/f"),
			("archive://:2:2//a/b.zip/c/d", "e/f", "archive://:4:1//a/b.zip/c/d/e/f"),
			// SFTP
			("sftp://remote//a", "b/c", "sftp://remote//a/b/c"),
			("sftp://remote:1:1//a/b/c", "d/e", "sftp://remote//a/b/c/d/e"),
			// Relative
			("search://kw", "b/c", "search://kw:2:2/b/c"),
			("search://kw/", "b/c", "search://kw:2:2/b/c"),
		];

		for (base, path, expected) in cases {
			let base: UrlBuf = base.parse()?;
			#[cfg(unix)]
			assert_eq!(format!("{:?}", base.join(path)), expected);
			#[cfg(windows)]
			assert_eq!(format!("{:?}", base.join(path)).replace(r"\", "/"), expected.replace(r"\", "/"));
		}

		Ok(())
	}

	#[test]
	fn test_parent() -> anyhow::Result<()> {
		crate::init_tests();
		let cases = [
			// Regular
			("/a", Some("/")),
			("/", None),
			// Search
			("search://kw:2:2//a/b/c", Some("search://kw:1:1//a/b")),
			("search://kw:1:1//a/b", Some("search://kw//a")),
			("search://kw//a", Some("/")),
			// Archive
			("archive://:2:1//a/b.zip/c/d", Some("archive://:1:1//a/b.zip/c")),
			("archive://:1:1//a/b.zip/c", Some("archive:////a/b.zip")),
			("archive:////a/b.zip", Some("/a")),
			// SFTP
			("sftp://remote:1:1//a/b", Some("sftp://remote//a")),
			("sftp://remote:1:1//a", Some("sftp://remote//")),
			("sftp://remote:1//", None),
			("sftp://remote//", None),
			// Relative
			("search://kw:2:2/a/b", Some("search://kw:1:1/a")),
			("search://kw:1:1/a", Some("search://kw/")),
			("search://kw/", None),
		];

		for (path, expected) in cases {
			let path: UrlBuf = path.parse()?;
			assert_eq!(path.parent().map(|u| format!("{u:?}")).as_deref(), expected);
		}

		Ok(())
	}

	#[test]
	fn test_into_search() -> Result<()> {
		crate::init_tests();
		const S: char = std::path::MAIN_SEPARATOR;

		let u: UrlBuf = "/root".parse()?;
		assert_eq!(format!("{u:?}"), "/root");

		let u = u.into_search("kw");
		assert_eq!(format!("{u:?}"), "search://kw//root");
		assert_eq!(format!("{:?}", u.parent().unwrap()), "/");

		let u = u.join("examples");
		assert_eq!(format!("{u:?}"), format!("search://kw:1:1//root{S}examples"));

		let u = u.join("README.md");
		assert_eq!(format!("{u:?}"), format!("search://kw:2:2//root{S}examples{S}README.md"));

		let u = u.parent().unwrap();
		assert_eq!(format!("{u:?}"), format!("search://kw:1:1//root{S}examples"));

		let u = u.parent().unwrap();
		assert_eq!(format!("{u:?}"), "search://kw//root");

		let u = u.parent().unwrap();
		assert_eq!(format!("{u:?}"), "/");

		Ok(())
	}
}
