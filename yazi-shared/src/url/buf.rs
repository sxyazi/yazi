use std::{borrow::Cow, fmt::{Debug, Formatter}, hash::{Hash, Hasher}, path::{Path, PathBuf}, str::FromStr};

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::{loc::LocBuf, path::{PathBufDyn, PathDynError, SetNameError}, pool::{InternStr, Pool, Symbol}, scheme::Scheme, strand::AsStrand, url::{AsUrl, Url, UrlCow, UrlLike}};

#[derive(Clone, Eq)]
pub enum UrlBuf {
	Regular(LocBuf),
	Search { loc: LocBuf, domain: Symbol<str> },
	Archive { loc: LocBuf, domain: Symbol<str> },
	Sftp { loc: LocBuf<typed_path::UnixPathBuf>, domain: Symbol<str> },
}

// FIXME: remove
impl Default for UrlBuf {
	fn default() -> Self { Self::Regular(Default::default()) }
}

impl From<&Self> for UrlBuf {
	fn from(url: &Self) -> Self { url.clone() }
}

impl From<Url<'_>> for UrlBuf {
	fn from(url: Url<'_>) -> Self {
		match url {
			Url::Regular(loc) => Self::Regular(loc.into()),
			Url::Search { loc, domain } => Self::Search { loc: loc.into(), domain: domain.intern() },
			Url::Archive { loc, domain } => Self::Archive { loc: loc.into(), domain: domain.intern() },
			Url::Sftp { loc, domain } => Self::Sftp { loc: loc.into(), domain: domain.intern() },
		}
	}
}

impl From<&Url<'_>> for UrlBuf {
	fn from(url: &Url<'_>) -> Self { (*url).into() }
}

impl From<LocBuf> for UrlBuf {
	fn from(loc: LocBuf) -> Self { Self::Regular(loc) }
}

impl From<PathBuf> for UrlBuf {
	fn from(path: PathBuf) -> Self { LocBuf::from(path).into() }
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

impl TryFrom<(Scheme, PathBufDyn)> for UrlBuf {
	type Error = anyhow::Error;

	fn try_from(value: (Scheme, PathBufDyn)) -> Result<Self, Self::Error> {
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
impl PartialEq for UrlBuf {
	fn eq(&self, other: &Self) -> bool { self.as_url() == other.as_url() }
}

impl PartialEq<UrlBuf> for &UrlBuf {
	fn eq(&self, other: &UrlBuf) -> bool { self.as_url() == other.as_url() }
}

impl PartialEq<Url<'_>> for UrlBuf {
	fn eq(&self, other: &Url) -> bool { self.as_url() == *other }
}

impl PartialEq<Url<'_>> for &UrlBuf {
	fn eq(&self, other: &Url) -> bool { self.as_url() == *other }
}

impl PartialEq<UrlCow<'_>> for UrlBuf {
	fn eq(&self, other: &UrlCow) -> bool { self.as_url() == other.as_url() }
}

impl PartialEq<UrlCow<'_>> for &UrlBuf {
	fn eq(&self, other: &UrlCow) -> bool { self.as_url() == other.as_url() }
}

// --- Hash
impl Hash for UrlBuf {
	fn hash<H: Hasher>(&self, state: &mut H) { self.as_url().hash(state) }
}

impl UrlBuf {
	#[inline]
	pub fn new() -> &'static Self {
		static U: UrlBuf = UrlBuf::Regular(LocBuf::empty());
		&U
	}

	#[inline]
	pub fn into_loc(self) -> PathBufDyn {
		match self {
			Self::Regular(loc) => loc.into_inner().into(),
			Self::Search { loc, .. } => loc.into_inner().into(),
			Self::Archive { loc, .. } => loc.into_inner().into(),
			Self::Sftp { loc, .. } => loc.into_inner().into(),
		}
	}

	#[inline]
	pub fn into_local(self) -> Option<PathBuf> {
		if self.kind().is_local() { self.into_loc().into_os().ok() } else { None }
	}

	pub fn try_set_name(&mut self, name: impl AsStrand) -> Result<(), SetNameError> {
		let name = name.as_strand();
		Ok(match self {
			Self::Regular(loc) => loc.try_set_name(name.as_os()?)?,
			Self::Search { loc, .. } => loc.try_set_name(name.as_os()?)?,
			Self::Archive { loc, .. } => loc.try_set_name(name.as_os()?)?,
			Self::Sftp { loc, .. } => loc.try_set_name(name.encoded_bytes())?,
		})
	}

	pub fn rebase(&self, base: &Path) -> Self {
		match self {
			Self::Regular(loc) => Self::Regular(loc.rebase(base)),
			Self::Search { loc, domain } => {
				Self::Search { loc: loc.rebase(base), domain: domain.clone() }
			}
			Self::Archive { loc, domain } => {
				Self::Archive { loc: loc.rebase(base), domain: domain.clone() }
			}
			Self::Sftp { loc, domain } => {
				todo!();
				// Self::Sftp { loc: loc.rebase(base), domain: domain.clone() }
			}
		}
	}
}

impl UrlBuf {
	#[inline]
	pub fn to_regular(&self) -> Result<Self, PathDynError> { Ok(self.as_url().as_regular()?.into()) }

	#[inline]
	pub fn into_regular(self) -> Result<Self, PathDynError> {
		Ok(Self::Regular(self.into_loc().into_os()?.into()))
	}

	#[inline]
	pub fn to_search(&self, domain: impl AsRef<str>) -> Result<Self, PathDynError> {
		Ok(Self::Search {
			loc:    LocBuf::<PathBuf>::zeroed(self.loc().to_os_owned()?),
			domain: Pool::<str>::intern(domain),
		})
	}

	#[inline]
	pub fn into_search(self, domain: impl AsRef<str>) -> Result<Self, PathDynError> {
		Ok(Self::Search {
			loc:    LocBuf::<PathBuf>::zeroed(self.into_loc().into_os()?),
			domain: Pool::<str>::intern(domain),
		})
	}
}

impl Debug for UrlBuf {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result { self.as_url().fmt(f) }
}

impl Serialize for UrlBuf {
	fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
		self.as_url().serialize(serializer)
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
			assert_eq!(format!("{:?}", base.try_join(path)?), expected);
			#[cfg(windows)]
			assert_eq!(
				format!("{:?}", base.try_join(path)?).replace(r"\", "/"),
				expected.replace(r"\", "/")
			);
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
			("sftp://remote:3:1//a/b", Some("sftp://remote//a")),
			("sftp://remote:2:1//a", Some("sftp://remote//")),
			("sftp://remote:1:1//a", Some("sftp://remote//")),
			("sftp://remote//a", Some("sftp://remote//")),
			("sftp://remote:1//", None),
			("sftp://remote//", None),
			// Relative
			("search://kw:2:2/a/b", Some("search://kw:1:1/a")),
			("search://kw:1:1/a", None),
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

		let u = u.into_search("kw")?;
		assert_eq!(format!("{u:?}"), "search://kw//root");
		assert_eq!(format!("{:?}", u.parent().unwrap()), "/");

		let u = u.try_join("examples")?;
		assert_eq!(format!("{u:?}"), format!("search://kw:1:1//root{S}examples"));

		let u = u.try_join("README.md")?;
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
