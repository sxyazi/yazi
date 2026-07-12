use std::{borrow::Cow, fmt::{Debug, Formatter}, hash::{Hash, Hasher}, path::{Path, PathBuf}, str::FromStr, sync::Arc};

use anyhow::Result;
use serde::{Deserialize, Serialize, de::{self, IntoDeserializer}};
use yazi_codegen::FromLuaOwned;
use yazi_macro::impl_data_any;

use crate::{auth::Auth, loc::LocBuf, path::{PathBufDyn, PathDynError, SetNameError}, spec::Spec, strand::AsStrand, url::{AsUrl, Url, UrlCow, UrlDeserializer, UrlLike}};

#[derive(Clone, Eq, FromLuaOwned)]
pub enum UrlBuf {
	Regular(LocBuf),
	Search { loc: LocBuf, auth: Arc<Auth> },
	Mount { loc: LocBuf, auth: Arc<Auth> },
	Scope { loc: LocBuf<typed_path::UnixPathBuf>, auth: Arc<Auth> },
	Sftp { loc: LocBuf<typed_path::UnixPathBuf>, auth: Arc<Auth> },
}

impl_data_any!(UrlBuf);

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
			Url::Search { loc, auth } => Self::Search { loc: loc.into(), auth: auth.clone() },
			Url::Mount { loc, auth } => Self::Mount { loc: loc.into(), auth: auth.clone() },
			Url::Scope { loc, auth } => Self::Scope { loc: loc.into(), auth: auth.clone() },
			Url::Sftp { loc, auth } => Self::Sftp { loc: loc.into(), auth: auth.clone() },
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

impl TryFrom<(Spec, PathBufDyn)> for UrlBuf {
	type Error = anyhow::Error;

	fn try_from(value: (Spec, PathBufDyn)) -> Result<Self, Self::Error> {
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
			Self::Mount { loc, .. } => loc.into_inner().into(),
			Self::Scope { loc, .. } => loc.into_inner().into(),
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
			Self::Mount { loc, .. } => loc.try_set_name(name.as_os()?)?,
			Self::Scope { loc, .. } => loc.try_set_name(name.encoded_bytes())?,
			Self::Sftp { loc, .. } => loc.try_set_name(name.encoded_bytes())?,
		})
	}

	pub fn rebase(&self, base: &Path) -> Self {
		match self {
			Self::Regular(loc) => Self::Regular(loc.rebase(base)),
			Self::Search { loc, auth } => Self::Search { loc: loc.rebase(base), auth: auth.clone() },
			Self::Mount { loc, auth } => Self::Mount { loc: loc.rebase(base), auth: auth.clone() },
			Self::Scope { .. } => todo!(),
			Self::Sftp { .. } => todo!(),
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
	pub fn into_search(self, query: impl AsRef<str>) -> Result<Self, PathDynError> {
		Ok(Self::Search {
			loc:  LocBuf::<PathBuf>::zeroed(self.into_loc().into_os()?),
			auth: Auth::search(query.as_ref()),
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
		struct Visitor;

		impl<'de> de::Visitor<'de> for Visitor {
			type Value = UrlBuf;

			fn expecting(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
				formatter.write_str("a Url or URL string")
			}

			fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
			where
				E: de::Error,
			{
				UrlBuf::from_str(value).map_err(E::custom)
			}

			fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
			where
				E: de::Error,
			{
				UrlBuf::try_from(value).map_err(E::custom)
			}

			fn visit_newtype_struct<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
			where
				D: de::Deserializer<'de>,
			{
				#[derive(Deserialize)]
				struct Shadow {
					#[serde(flatten)]
					spec: Spec,
					path: Vec<u8>,
				}

				let Shadow { spec, path } = Deserialize::deserialize(deserializer)?;
				let path = PathBufDyn::with(spec.kind, path).map_err(de::Error::custom)?;

				UrlBuf::try_from((spec, path)).map_err(de::Error::custom)
			}
		}

		deserializer.deserialize_string(Visitor)
	}
}

impl<'de> IntoDeserializer<'de, de::value::Error> for UrlBuf {
	type Deserializer = UrlDeserializer<'de>;

	fn into_deserializer(self) -> Self::Deserializer { UrlDeserializer(self.into()) }
}

impl<'de> IntoDeserializer<'de, de::value::Error> for &'de UrlBuf {
	type Deserializer = UrlDeserializer<'de>;

	fn into_deserializer(self) -> Self::Deserializer { UrlDeserializer(self.into()) }
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
			// Mount
			("test-mount://7z//a/b.zip", "c/d", "test-mount://7z:2:1//a/b.zip/c/d"),
			("test-mount://7z:2:1//a/b.zip/c/d", "e/f", "test-mount://7z:4:1//a/b.zip/c/d/e/f"),
			("test-mount://7z:2:2//a/b.zip/c/d", "e/f", "test-mount://7z:4:1//a/b.zip/c/d/e/f"),
			// SFTP
			("sftp://vps//a", "b/c", "sftp://vps//a/b/c"),
			("sftp://vps:1:1//a/b/c", "d/e", "sftp://vps//a/b/c/d/e"),
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
			// Mount
			("test-mount://7z:2:1//a/b.zip/c/d", Some("test-mount://7z:1:1//a/b.zip/c")),
			("test-mount://7z:1:1//a/b.zip/c", Some("test-mount://7z//a/b.zip")),
			("test-mount://7z//a/b.zip", Some("/a")),
			// SFTP
			("sftp://vps:3:1//a/b", Some("sftp://vps//a")),
			("sftp://vps:2:1//a", Some("sftp://vps//")),
			("sftp://vps:1:1//a", Some("sftp://vps//")),
			("sftp://vps//a", Some("sftp://vps//")),
			("sftp://vps:1//", None),
			("sftp://vps//", None),
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
