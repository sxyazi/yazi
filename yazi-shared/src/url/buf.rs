use std::{borrow::Cow, fmt::Formatter, hash::{Hash, Hasher}, path::{Path, PathBuf}, str::FromStr};

use anyhow::{Result, ensure};
use serde::{Deserialize, Serialize, de::{self, IntoDeserializer}};
use yazi_macro::impl_data_any;

use crate::{auth::{AuthArc, View}, domain::Domain, loc::LocBuf, path::{PathBufDyn, PathDynError, SetNameError}, spec::Spec, strand::AsStrand, url::{AsUrl, Url, UrlCow, UrlDeserializer, UrlLike}, wire::Wire};

#[derive(Clone, Debug, Eq)]
pub enum UrlBuf {
	Os { loc: LocBuf, auth: AuthArc },
	Unix { loc: LocBuf<typed_path::UnixPathBuf>, auth: AuthArc },
}

impl_data_any!(UrlBuf);

// FIXME: remove
impl Default for UrlBuf {
	fn default() -> Self { Self::Os { loc: Default::default(), auth: Default::default() } }
}

impl From<&Self> for UrlBuf {
	fn from(url: &Self) -> Self { url.clone() }
}

impl From<Url<'_>> for UrlBuf {
	fn from(url: Url<'_>) -> Self {
		match url {
			Url::Os { loc, auth } => Self::Os { loc: loc.into(), auth: auth.clone() },
			Url::Unix { loc, auth } => Self::Unix { loc: loc.into(), auth: auth.clone() },
		}
	}
}

impl From<&Url<'_>> for UrlBuf {
	fn from(url: &Url<'_>) -> Self { (*url).into() }
}

impl From<LocBuf> for UrlBuf {
	fn from(loc: LocBuf) -> Self { Self::Os { loc, auth: Default::default() } }
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
	pub fn into_loc(self) -> PathBufDyn {
		match self {
			Self::Os { loc, .. } => loc.into_inner().into(),
			Self::Unix { loc, .. } => loc.into_inner().into(),
		}
	}

	pub fn into_view(self, auth: AuthArc, data: Wire) -> Result<Self> {
		ensure!(auth.kind.is_view(), "View authority required");

		let source = self.physical().auth().clone();
		let auth = auth.with_view(View { source, data });
		auth.validate()?;

		Ok(match self {
			Self::Os { loc, .. } => Self::Os { loc: LocBuf::zeroed(loc.into_inner()), auth },
			Self::Unix { loc, .. } => Self::Unix { loc: LocBuf::zeroed(loc.into_inner()), auth },
		})
	}

	pub fn into_physical(self) -> Self {
		let Some(auth) = self.auth().view.auth().cloned() else { return self };

		match self {
			Self::Os { loc, .. } => Self::Os { loc: loc.into_inner().into(), auth },
			Self::Unix { loc, .. } => Self::Unix { loc: loc.into_inner().into(), auth },
		}
	}

	pub fn try_set_name(&mut self, name: impl AsStrand) -> Result<(), SetNameError> {
		let name = name.as_strand();
		Ok(match self {
			Self::Os { loc, .. } => loc.try_set_name(name.as_os()?)?,
			Self::Unix { loc, .. } => loc.try_set_name(name.encoded_bytes())?,
		})
	}

	pub fn rebase(&self, base: &Path) -> Self {
		match self {
			Self::Os { loc, auth } => Self::Os { loc: loc.rebase(base), auth: auth.clone() },
			Self::Unix { .. } => todo!(),
		}
	}
}

impl UrlBuf {
	#[inline]
	pub fn to_regular(&self) -> Result<Self, PathDynError> { Ok(self.as_url().as_regular()?.into()) }

	#[inline]
	pub(crate) fn with_domain<'a>(mut self, domain: impl Into<Domain<'a>>) -> Self {
		match &mut self {
			Self::Os { auth, .. } | Self::Unix { auth, .. } if !auth.is_regular() => {
				auth.make_mut().domain = domain.into().into_owned()
			}
			_ => {}
		}
		self
	}
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

			fn visit_map<A>(self, map: A) -> Result<Self::Value, A::Error>
			where
				A: de::MapAccess<'de>,
			{
				self.visit_newtype_struct(de::value::MapAccessDeserializer::new(map))
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

				let Shadow { spec, path } = Shadow::deserialize(deserializer)?;
				let kind = spec.path_kind().map_err(de::Error::custom)?;
				let path = PathBufDyn::with(kind, path).map_err(de::Error::custom)?;

				UrlBuf::try_from((spec, path)).map_err(de::Error::custom)
			}
		}

		deserializer.deserialize_any(Visitor)
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
	use std::fmt::Display;

	use anyhow::Result;

	use super::*;
	use crate::{path::PathKind, url::UrlLike};

	fn fmt(value: impl Display) -> String { format!("{value}").replace(r"\", "/") }

	#[test]
	fn test_join() -> anyhow::Result<()> {
		crate::init_tests();
		let cases = [
			// Regular
			("/a", "b/c", "/a/b/c"),
			// View
			("test-view://fx/@Ds2kw0A//a", "b/c", "test-view://fx:2:2/@Ds2kw0A//a/b/c"),
			("test-view://fx:2:2/@Ds2kw0A//a/b/c", "d/e", "test-view://fx:4:4/@Ds2kw0A//a/b/c/d/e"),
			// Mount
			("test-mount://7z//a/b.zip", "c/d", "test-mount://7z:2:1//a/b.zip/c/d"),
			("test-mount://7z:2:1//a/b.zip/c/d", "e/f", "test-mount://7z:4:1//a/b.zip/c/d/e/f"),
			("test-mount://7z:2:2//a/b.zip/c/d", "e/f", "test-mount://7z:4:1//a/b.zip/c/d/e/f"),
			// SFTP
			("sftp://vps//a", "b/c", "sftp://vps//a/b/c"),
			("sftp://vps:1:1//a/b/c", "d/e", "sftp://vps//a/b/c/d/e"),
			// Relative view
			("test-view://fx/@Ds2kw0A/", "b/c", "test-view://fx:2:2/@Ds2kw0A/b/c"),
		];

		for (base, path, expected) in cases {
			let base: UrlBuf = base.parse()?;
			assert_eq!(fmt(base.try_join(path)?), expected);
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
			// View
			("test-view://fx:2:2/@Ds2kw0A//a/b/c", Some("test-view://fx:1:1/@Ds2kw0A//a/b")),
			("test-view://fx:1:1/@Ds2kw0A//a/b", Some("test-view://fx/@Ds2kw0A//a")),
			("test-view://fx/@Ds2kw0A//a", Some("/")),
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
			// Relative view
			("test-view://fx:2:2/@Ds2kw0A/a/b", Some("test-view://fx:1:1/@Ds2kw0A/a")),
			("test-view://fx:1:1/@Ds2kw0A/a", Some("test-view://fx/@Ds2kw0A/")),
			("test-view://fx/@Ds2kw0A/", None),
			("test-mount://7z:1:1/a", Some("test-mount://7z/")),
			("test-scope://aws/a", Some("test-scope://aws/")),
			("sftp://vps/a", Some("sftp://vps/")),
		];

		for (path, expected) in cases {
			let path: UrlBuf = path.parse()?;
			assert_eq!(path.parent().map(fmt).as_deref(), expected);
		}

		Ok(())
	}

	#[test]
	fn test_view() -> Result<()> {
		crate::init_tests();

		let u: UrlBuf = "test-view://fx/@Ds2kw0A//root".parse()?;
		assert_eq!(fmt(&u), "test-view://fx/@Ds2kw0A//root");
		assert_eq!(fmt(u.physical()), "/root");
		assert_eq!(fmt(u.parent().unwrap()), "/");

		let u = u.try_join("examples")?;
		assert_eq!(fmt(&u), "test-view://fx:1:1/@Ds2kw0A//root/examples");
		assert_eq!(u.urn(), "examples");
		assert_eq!(fmt(u.physical()), "/root/examples");

		let u = u.try_join("README.md")?;
		assert_eq!(fmt(&u), "test-view://fx:2:2/@Ds2kw0A//root/examples/README.md");
		assert_eq!(u.urn(), "examples/README.md");
		assert_eq!(fmt(u.physical()), "/root/examples/README.md");

		let u = u.parent().unwrap();
		assert_eq!(fmt(u), "test-view://fx:1:1/@Ds2kw0A//root/examples");
		assert_eq!(u.urn(), "examples");

		let u = u.parent().unwrap();
		assert_eq!(fmt(u), "test-view://fx/@Ds2kw0A//root");
		assert!(u.urn().is_empty());

		let u = u.parent().unwrap();
		assert_eq!(fmt(u), "/");

		Ok(())
	}

	#[test]
	fn test_hub_parse() -> Result<()> {
		crate::init_tests();

		let root: UrlBuf = "test-hub://root/@/".parse()?;
		assert_eq!(fmt(&root), "test-hub://root/@/");
		assert_eq!(root.loc().kind(), PathKind::Os);

		let encoded: UrlBuf = "test-hub://%252C/@/".parse()?;
		assert_eq!(fmt(encoded.encode()), "test-hub~://%252C/@/");
		let encoded: UrlBuf = "test-hub://b1/@a%2Cb%40c%25d%2Fe,root/foo/bar".parse()?;
		assert_eq!(fmt(encoded.encode()), "test-hub~://b1/@a%2Cb%40c%25d%2Fe,root/foo/bar");

		Ok(())
	}

	#[test]
	fn test_hub_domain() -> Result<()> {
		crate::init_tests();

		let root: UrlBuf = "test-hub://root/@/".parse()?;
		let foo = root.try_join("foo")?.with_domain("a1");
		assert_eq!(fmt(&foo), "test-hub://a1/@root/foo");

		let bar = foo.try_join("bar")?.with_domain("b1");
		assert_eq!(bar.key(), "b1");
		assert_eq!(fmt(&bar), "test-hub://b1/@a1,root/foo/bar");
		assert_eq!(fmt(bar.parent().unwrap()), "test-hub://a1/@root/foo");
		assert_eq!(fmt(bar.parent().unwrap().parent().unwrap()), "test-hub://root/@/");

		let relative = UrlCow::try_from("test-hub://a1/@/@abc")?;
		assert_eq!(fmt(&relative), "test-hub://a1/@/@abc");
		assert_eq!(fmt(relative.parent().unwrap()), "test-hub:///@/");
		assert_eq!(relative.key(), "a1");
		assert!(!relative.is_owned());
		assert!(relative.parent().unwrap().key().is_empty());

		Ok(())
	}

	#[test]
	fn test_hub_join() -> Result<()> {
		crate::init_tests();

		let root: UrlBuf = "test-hub://root/@/".parse()?;
		let bar: UrlBuf = "test-hub://b1/@a1,root/foo/bar".parse()?;

		assert_eq!(fmt(bar.try_join(".")?), "test-hub://b1/@a1,root/foo/bar");
		assert_eq!(fmt(bar.try_join("..")?), "test-hub:///@b1,a1,root/foo/bar/..");

		assert_eq!(fmt(bar.try_join("/x/y")?), "test-hub:///@,//x/y");
		assert_eq!(fmt(root.try_join("../../..")?), "test-hub:///@,,root/../../..");

		let absolute = root.try_join("/foo")?;
		assert_eq!(fmt(&absolute), "test-hub:///@//foo");
		let absolute = absolute.with_domain("a1");
		assert_eq!(fmt(&absolute), "test-hub://a1/@//foo");
		assert_eq!(fmt(absolute.parent().unwrap().try_join("..")?), "test-hub:///@//..");

		Ok(())
	}

	#[test]
	fn test_hub_ports() -> Result<()> {
		crate::init_tests();

		let ports: UrlBuf = "test-hub://b1:2:1/@a1,root/foo/bar".parse()?;
		assert_eq!(fmt(ports.base()), "test-hub://root/@/");
		assert_eq!(fmt(ports.trail()), "test-hub://a1/@root/foo");

		let ports: UrlBuf = "test-hub://b1:3:1/@a1,root//foo/bar".parse()?;
		assert_eq!(fmt(ports.base()), "test-hub://root/@/");
		assert_eq!(fmt(ports.trail()), "test-hub://a1/@root//foo");

		let zeroed: UrlBuf = "test-hub://b1:0:0/@a1,root/foo/bar".parse()?;
		assert_eq!(fmt(zeroed.base()), "test-hub://b1/@a1,root/foo/bar");
		assert_eq!(fmt(zeroed.trail()), "test-hub://b1/@a1,root/foo/bar");

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
			fmt(url.try_replace(2, Path::new("baz/qux"))?),
			"test-hub://b1:2:2/@,a1,root/foo/baz/qux"
		);
		assert_eq!(fmt(url.try_replace(1, Path::new("qux"))?), "test-hub://b1/@root/qux");

		Ok(())
	}

	#[cfg(windows)]
	#[test]
	fn test_hub_windows() -> Result<()> {
		crate::init_tests();

		let root: UrlBuf = "test-hub://root/@/".parse()?;
		let c = root.try_join(r"C:\")?.with_domain("c-root");
		assert_eq!(c.key(), "c-root");
		assert_eq!(c.auth().parent_depth(), 0);

		let drive = c.try_join(r"Users\file.txt")?.with_domain("file");
		assert_eq!(drive.loc(), Path::new(r"C:\Users\file.txt"));
		assert_eq!(drive.auth().parent_depth(), 2);

		let parent = drive.parent().unwrap();
		assert_eq!(parent.loc(), Path::new(r"C:\Users"));

		let parent = parent.parent().unwrap();
		assert_eq!(parent.loc(), Path::new(r"C:\"));
		assert_eq!(parent.key(), "c-root");
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

	#[test]
	fn test_regular_root_key() -> Result<()> {
		crate::init_tests();

		let root: UrlBuf = "/".parse()?;
		assert!(root.key().is_empty());
		assert!(root.pair().is_none());

		let child: UrlBuf = "/foo".parse()?;
		assert_eq!(child.key(), "foo");
		assert!(!child.key().has_root());

		Ok(())
	}
}
