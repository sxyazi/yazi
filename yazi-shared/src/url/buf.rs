use std::{borrow::Cow, fmt::{Debug, Formatter}, hash::{Hash, Hasher}, path::{Path, PathBuf}, str::FromStr, sync::Arc};

use anyhow::Result;
use serde::{Deserialize, Serialize, de::{self, IntoDeserializer}};
use yazi_codegen::FromLuaOwned;
use yazi_macro::impl_data_any;

use crate::{auth::{Auth, Domain}, loc::LocBuf, path::{PathBufDyn, PathDynError, SetNameError}, spec::Spec, strand::AsStrand, url::{AsUrl, Url, UrlCow, UrlDeserializer, UrlLike}};

#[derive(Clone, Eq, FromLuaOwned)]
pub enum UrlBuf {
	Regular(LocBuf),
	Search { loc: LocBuf, auth: Arc<Auth> },
	Mount { loc: LocBuf, auth: Arc<Auth> },
	Hub { loc: LocBuf, auth: Arc<Auth> },
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
			Url::Hub { loc, auth } => Self::Hub { loc: loc.into(), auth: auth.clone() },
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
			Self::Hub { loc, .. } => loc.into_inner().into(),
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
			Self::Hub { loc, .. } => loc.try_set_name(name.as_os()?)?,
			Self::Scope { loc, .. } => loc.try_set_name(name.encoded_bytes())?,
			Self::Sftp { loc, .. } => loc.try_set_name(name.encoded_bytes())?,
		})
	}

	pub fn rebase(&self, base: &Path) -> Self {
		match self {
			Self::Regular(loc) => Self::Regular(loc.rebase(base)),
			Self::Search { loc, auth } => Self::Search { loc: loc.rebase(base), auth: auth.clone() },
			Self::Mount { loc, auth } => Self::Mount { loc: loc.rebase(base), auth: auth.clone() },
			Self::Hub { .. } => todo!(),
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

	pub fn into_domain<'a>(mut self, domain: impl Into<Domain<'a>>) -> Self {
		match &mut self {
			Self::Regular(_) => {}
			Self::Search { auth, .. }
			| Self::Mount { auth, .. }
			| Self::Hub { auth, .. }
			| Self::Scope { auth, .. }
			| Self::Sftp { auth, .. } => Arc::make_mut(auth).domain = domain.into().into_owned(),
		}
		self
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
	use typed_path::UnixPath;

	use super::*;
	use crate::{path::PathKind, url::UrlLike};

	const S: char = std::path::MAIN_SEPARATOR;

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
			("a", Some("")),
			("", None),
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
			("search://kw:1:1/a", Some("search://kw/")),
			("search://kw/", None),
			("test-mount://7z:1:1/a", Some("test-mount://7z/")),
			("test-scope://aws/a", Some("test-scope://aws/")),
			("sftp://vps/a", Some("sftp://vps/")),
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

	#[test]
	fn test_hub_parse() -> Result<()> {
		crate::init_tests();

		let root: UrlBuf = "test-hub://root/@/".parse()?;
		assert_eq!(format!("{root:?}"), "test-hub://root/@/");
		assert_eq!(root.loc().kind(), PathKind::Os);

		let encoded: UrlBuf = "test-hub://%252C/@/".parse()?;
		assert_eq!(format!("{encoded:?}"), "test-hub://%252C/@/");
		let encoded: UrlBuf = "test-hub://b1/@a%2Cb%40c%25d%2Fe,root/foo/bar".parse()?;
		assert_eq!(format!("{encoded:?}"), format!("test-hub://b1/@a%2Cb%40c%25d%2Fe,root/foo{S}bar"));

		Ok(())
	}

	#[test]
	fn test_hub_domain() -> Result<()> {
		crate::init_tests();

		let root: UrlBuf = "test-hub://root/@/".parse()?;
		let foo = root.try_join("foo")?.into_domain("a1");
		assert_eq!(format!("{foo:?}"), "test-hub://a1/@root/foo");

		let bar = foo.try_join("bar")?.into_domain("b1");
		assert_eq!(bar.entry_key(), "b1");
		assert_eq!(format!("{bar:?}"), format!("test-hub://b1/@a1,root/foo{S}bar"));
		assert_eq!(format!("{:?}", bar.parent().unwrap()), "test-hub://a1/@root/foo");
		assert_eq!(format!("{:?}", bar.parent().unwrap().parent().unwrap()), "test-hub://root/@/");

		let relative = UrlCow::try_from("test-hub://a1/@/@abc")?;
		assert_eq!(format!("{:?}", relative.as_url()), "test-hub://a1/@/@abc");
		assert_eq!(format!("{:?}", relative.parent().unwrap().as_url()), "test-hub:///@/");
		assert_eq!(relative.entry_key(), "a1");
		assert!(!relative.is_owned());
		assert!(relative.parent().unwrap().entry_key().is_empty());

		Ok(())
	}

	#[test]
	fn test_hub_join() -> Result<()> {
		crate::init_tests();

		let root: UrlBuf = "test-hub://root/@/".parse()?;
		let bar: UrlBuf = "test-hub://b1/@a1,root/foo/bar".parse()?;

		assert_eq!(format!("{:?}", bar.try_join(".")?), format!("test-hub://b1/@a1,root/foo{S}bar"));
		assert_eq!(
			format!("{:?}", bar.try_join("..")?),
			format!("test-hub:///@b1,a1,root/foo{S}bar{S}..")
		);

		assert_eq!(format!("{:?}", bar.try_join("/x/y")?), format!("test-hub:///@,/{S}x{S}y"));
		assert_eq!(
			format!("{:?}", root.try_join("../../..")?),
			format!("test-hub:///@,,root/..{S}..{S}..")
		);

		let absolute = root.try_join("/foo")?;
		assert_eq!(format!("{absolute:?}"), format!("test-hub:///@/{S}foo"));
		let absolute = absolute.into_domain("a1");
		assert_eq!(format!("{absolute:?}"), format!("test-hub://a1/@/{S}foo"));
		assert_eq!(
			format!("{:?}", absolute.parent().unwrap().try_join("..")?),
			format!("test-hub:///@/{S}..")
		);

		Ok(())
	}

	#[test]
	fn test_hub_ports() -> Result<()> {
		crate::init_tests();

		let ports: UrlBuf = "test-hub://b1:2:1/@a1,root/foo/bar".parse()?;
		assert_eq!(format!("{:?}", ports.base()), "test-hub://root/@/");
		assert_eq!(format!("{:?}", ports.trail()), "test-hub://a1/@root/foo");

		let ports: UrlBuf = "test-hub://b1:3:1/@a1,root//foo/bar".parse()?;
		assert_eq!(format!("{:?}", ports.base()), "test-hub://root/@/");
		assert_eq!(format!("{:?}", ports.trail()), format!("test-hub://a1/@root/{S}foo"));

		let zeroed: UrlBuf = "test-hub://b1:0:0/@a1,root/foo/bar".parse()?;
		assert_eq!(format!("{:?}", zeroed.base()), format!("test-hub://b1/@a1,root/foo{S}bar"));
		assert_eq!(format!("{:?}", zeroed.trail()), format!("test-hub://b1/@a1,root/foo{S}bar"));

		Ok(())
	}

	#[test]
	fn test_hub_invalid() {
		crate::init_tests();

		assert!("test-hub://a1/foo".parse::<UrlBuf>().is_err());
		assert!("test-hub://b1/@a1/foo/bar".parse::<UrlBuf>().is_err());
	}

	#[test]
	fn test_hub_replace() -> Result<()> {
		crate::init_tests();

		let url: UrlBuf = "test-hub://b1/@a1,root/foo/bar".parse()?;
		assert_eq!(
			format!("{:?}", url.try_replace(2, Path::new("baz/qux"))?.as_url()),
			format!("test-hub://b1:2:2/@,a1,root/foo{S}baz{S}qux")
		);
		assert_eq!(
			format!("{:?}", url.try_replace(1, Path::new("qux"))?.as_url()),
			"test-hub://b1/@root/qux"
		);

		Ok(())
	}

	#[cfg(windows)]
	#[test]
	fn test_hub_windows() -> Result<()> {
		crate::init_tests();

		let root: UrlBuf = "test-hub://root/@/".parse()?;
		let c = root.try_join(r"C:\")?.into_domain("c-root");
		assert_eq!(c.entry_key(), "c-root");
		assert_eq!(c.auth().parent_depth(), 0);

		let drive = c.try_join(r"Users\file.txt")?.into_domain("file");
		assert_eq!(drive.loc(), Path::new(r"C:\Users\file.txt"));
		assert_eq!(drive.auth().parent_depth(), 2);

		let parent = drive.parent().unwrap();
		assert_eq!(parent.loc(), Path::new(r"C:\Users"));

		let parent = parent.parent().unwrap();
		assert_eq!(parent.loc(), Path::new(r"C:\"));
		assert_eq!(parent.entry_key(), "c-root");
		assert!(parent.parent().is_none());

		let relative = root.try_join(r"C:foo")?;
		assert!(!relative.is_absolute());
		assert_eq!(relative.parent().unwrap().loc(), Path::new("C:"));
		assert!(relative.parent().unwrap().parent().is_none());

		let unc = root.try_join(r"\\server\share\dir\file")?;
		assert!(unc.is_absolute());
		assert_eq!(unc.loc(), Path::new(r"\\server\share\dir\file"));
		assert_eq!(unc.auth().parent_depth(), 2);

		Ok(())
	}
}
