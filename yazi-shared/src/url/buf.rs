use std::{borrow::Cow, ffi::OsStr, fmt::{Debug, Formatter}, path::{Path, PathBuf}, str::FromStr, sync::OnceLock};

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::{loc::LocBuf, pool::Pool, scheme::{Scheme, SchemeLike}, url::{AsUrl, Encode, EncodeTilded, Url, UrlCow}};

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
	pub fn new() -> &'static Self {
		static U: OnceLock<UrlBuf> = OnceLock::new();
		U.get_or_init(Self::default)

		// FIXME: use `LocBuf::empty()` when Rust 1.91.0 released
		// static U: UrlBuf = UrlBuf { loc: LocBuf::empty(), scheme: Scheme::Regular
		// }; &U
	}

	#[inline]
	pub fn into_path(self) -> Option<PathBuf> {
		Some(self.loc.into_path()).filter(|_| self.scheme.is_local())
	}

	#[inline]
	pub fn set_name(&mut self, name: impl AsRef<OsStr>) { self.loc.set_name(name); }

	#[inline]
	pub fn rebase(&self, base: &Path) -> Self {
		Self { loc: self.loc.rebase(base), scheme: self.scheme.clone() }
	}
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
	use crate::url::UrlLike;

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
