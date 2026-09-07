use std::{borrow::Cow, hash::{Hash, Hasher}, path::PathBuf};

use anyhow::{Result, ensure};
use serde::{Deserialize, Deserializer, Serialize};
use typed_path::{UnixPath, UnixPathBuf};

use crate::{auth::AuthArc, loc::{Loc, LocBuf, LocCow}, path::{DynPath, PathBufDyn, PathCow, PathDyn}, spec::Spec, url::{AsUrl, Url, UrlBuf, UrlLike}};

#[derive(Clone, Debug)]
pub enum UrlCow<'a> {
	Os { loc: LocCow<'a>, auth: AuthArc },
	Unix { loc: LocCow<'a, &'a UnixPath, UnixPathBuf>, auth: AuthArc },
}

impl<'a> From<Url<'a>> for UrlCow<'a> {
	fn from(value: Url<'a>) -> Self {
		match value {
			Url::Os { loc, auth } => Self::Os { loc: loc.into(), auth: auth.clone() },
			Url::Unix { loc, auth } => Self::Unix { loc: loc.into(), auth: auth.clone() },
		}
	}
}

impl<'a, T> From<&'a T> for UrlCow<'a>
where
	T: AsUrl + ?Sized,
{
	fn from(value: &'a T) -> Self { value.as_url().into() }
}

impl From<UrlBuf> for UrlCow<'_> {
	fn from(value: UrlBuf) -> Self {
		match value {
			UrlBuf::Os { loc, auth } => Self::Os { loc: loc.into(), auth },
			UrlBuf::Unix { loc, auth } => Self::Unix { loc: loc.into(), auth },
		}
	}
}

impl From<PathBuf> for UrlCow<'_> {
	fn from(value: PathBuf) -> Self { UrlBuf::from(value).into() }
}

impl From<UrlCow<'_>> for UrlBuf {
	fn from(value: UrlCow<'_>) -> Self { value.into_owned() }
}

impl From<&UrlCow<'_>> for UrlBuf {
	fn from(value: &UrlCow<'_>) -> Self { value.as_url().into() }
}

impl<'a> TryFrom<&'a [u8]> for UrlCow<'a> {
	type Error = anyhow::Error;

	fn try_from(value: &'a [u8]) -> Result<Self, Self::Error> { Spec::parse(value)?.try_into() }
}

impl TryFrom<Vec<u8>> for UrlCow<'_> {
	type Error = anyhow::Error;

	fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
		Ok(UrlCow::try_from(value.as_slice())?.into_owned().into())
	}
}

impl<'a> TryFrom<&'a str> for UrlCow<'a> {
	type Error = anyhow::Error;

	fn try_from(value: &'a str) -> Result<Self, Self::Error> { Self::try_from(value.as_bytes()) }
}

impl TryFrom<String> for UrlCow<'_> {
	type Error = anyhow::Error;

	fn try_from(value: String) -> Result<Self, Self::Error> {
		Ok(UrlCow::try_from(value.as_str())?.into_owned().into())
	}
}

impl<'a> TryFrom<Cow<'a, str>> for UrlCow<'a> {
	type Error = anyhow::Error;

	fn try_from(value: Cow<'a, str>) -> Result<Self, Self::Error> {
		match value {
			Cow::Borrowed(s) => UrlCow::try_from(s),
			Cow::Owned(s) => UrlCow::try_from(s),
		}
	}
}

impl<'a> TryFrom<(Spec, PathCow<'a>)> for UrlCow<'a> {
	type Error = anyhow::Error;

	fn try_from((spec, path): (Spec, PathCow<'a>)) -> Result<Self, Self::Error> {
		match path {
			PathCow::Borrowed(path) => (spec, path).try_into(),
			PathCow::Owned(path) => (spec, path).try_into(),
		}
	}
}

impl<'a> TryFrom<(Spec, PathDyn<'a>)> for UrlCow<'a> {
	type Error = anyhow::Error;

	fn try_from((spec, path): (Spec, PathDyn<'a>)) -> Result<Self, Self::Error> {
		let Spec { auth, uri, urn } = spec;
		validate_auth_path(&auth, path)?;

		Ok(match path {
			PathDyn::Os(path) if auth.is_regular() => Self::Os { loc: Loc::bare(path).into(), auth },
			PathDyn::Os(path) => Self::Os { loc: Loc::with(path, uri, urn)?.into(), auth },
			PathDyn::Unix(path) => Self::Unix { loc: Loc::with(path, uri, urn)?.into(), auth },
		})
	}
}

impl<'a> TryFrom<(Spec, PathBufDyn)> for UrlCow<'a> {
	type Error = anyhow::Error;

	fn try_from((spec, path): (Spec, PathBufDyn)) -> Result<Self, Self::Error> {
		let Spec { auth, uri, urn } = spec;
		validate_auth_path(&auth, path.dyn_path())?;

		Ok(match path {
			PathBufDyn::Os(path) if auth.is_regular() => {
				Self::Os { loc: LocBuf::from(path).into(), auth }
			}
			PathBufDyn::Os(path) => Self::Os { loc: LocBuf::with(path, uri, urn)?.into(), auth },
			PathBufDyn::Unix(path) => Self::Unix { loc: LocBuf::with(path, uri, urn)?.into(), auth },
		})
	}
}

// --- Eq
impl PartialEq for UrlCow<'_> {
	fn eq(&self, other: &Self) -> bool { self.as_url() == other.as_url() }
}

impl PartialEq<UrlBuf> for UrlCow<'_> {
	fn eq(&self, other: &UrlBuf) -> bool { self.as_url() == other.as_url() }
}

impl Eq for UrlCow<'_> {}

// --- Hash
impl Hash for UrlCow<'_> {
	fn hash<H: Hasher>(&self, state: &mut H) { self.as_url().hash(state); }
}

impl<'a> UrlCow<'a> {
	pub fn is_owned(&self) -> bool {
		match self {
			Self::Os { loc, .. } => loc.is_owned(),
			Self::Unix { loc, .. } => loc.is_owned(),
		}
	}

	pub fn into_owned(self) -> UrlBuf {
		match self {
			Self::Os { loc, auth } => UrlBuf::Os { loc: loc.into_owned(), auth },
			Self::Unix { loc, auth } => UrlBuf::Unix { loc: loc.into_owned(), auth },
		}
	}

	pub fn into_pair(self) -> (Spec, PathCow<'a>) {
		let (uri, urn) = Spec::retrieve_ports(self.as_url());
		let (auth, path) = match self {
			Self::Os { loc, auth } => (auth, loc.into_path()),
			Self::Unix { loc, auth } => (auth, loc.into_path()),
		};
		(Spec { auth, uri, urn }, path)
	}

	pub fn with_ports(self, uri: usize, urn: usize) -> Result<Self> {
		let url = match self {
			Self::Os { loc, auth } => Self::Os { loc: loc.with_ports(uri, urn)?, auth },
			Self::Unix { loc, auth } => Self::Unix { loc: loc.with_ports(uri, urn)?, auth },
		};

		validate_auth_path(url.auth(), url.loc())?;
		Ok(url)
	}

	pub fn to_owned(&self) -> UrlBuf { self.as_url().into() }
}

impl Serialize for UrlCow<'_> {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		self.as_url().serialize(serializer)
	}
}

impl<'de> Deserialize<'de> for UrlCow<'_> {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		UrlBuf::deserialize(deserializer).map(UrlCow::from)
	}
}

fn validate_auth_path(auth: &AuthArc, path: PathDyn) -> Result<()> {
	auth.validate()?;
	ensure!(auth.path_kind()? == path.kind(), "URL path kind does not match Auth kind");

	if auth.kind.is_hub() {
		ensure!(
			auth.parent_depth() == path.components().auth_depth(),
			"Hub URL parent depth does not match its path"
		);
	}
	Ok(())
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::url::UrlLike;

	#[test]
	fn test_parse() -> Result<()> {
		crate::init_tests();

		struct Case {
			url:   &'static str,
			urn:   &'static str,
			uri:   &'static str,
			trail: &'static str,
			base:  &'static str,
		}

		let cases = [
			// Regular
			Case {
				url:   "/root/music/rock/song.mp3",
				urn:   "song.mp3",
				uri:   "song.mp3",
				trail: "/root/music/rock/",
				base:  "/root/music/rock/",
			},
			// View portal
			Case {
				url:   "test-view://fx/@Ds2kw0A//root/Documents/reports",
				urn:   "",
				uri:   "",
				trail: "test-view://fx/@Ds2kw0A//root/Documents/reports",
				base:  "test-view://fx/@Ds2kw0A//root/Documents/reports",
			},
			// View item
			Case {
				url:   "test-view://fx:2:2/@Ds2kw0A//root/Documents/reports/2023/summary.docx",
				urn:   "2023/summary.docx",
				uri:   "2023/summary.docx",
				trail: "test-view://fx/@Ds2kw0A//root/Documents/reports/",
				base:  "test-view://fx/@Ds2kw0A//root/Documents/reports/",
			},
			// Mount portal
			Case {
				url:   "test-mount://7z//root/Downloads/images.zip",
				urn:   "",
				uri:   "",
				trail: "test-mount://7z//root/Downloads/images.zip",
				base:  "test-mount://7z//root/Downloads/images.zip",
			},
			// Mount item
			Case {
				url:   "test-mount://7z:2:1//root/Downloads/images.zip/2025/city.jpg",
				urn:   "city.jpg",
				uri:   "2025/city.jpg",
				trail: "test-mount://7z:1:1//root/Downloads/images.zip/2025/",
				base:  "test-mount://7z//root/Downloads/images.zip/",
			},
			// SFTP
			Case {
				url:   "sftp://vps//root/docs/report.pdf",
				urn:   "report.pdf",
				uri:   "report.pdf",
				trail: "sftp://vps//root/docs/",
				base:  "sftp://vps//root/docs/",
			},
		];

		for case in cases {
			let url = UrlCow::try_from(case.url)?;
			assert_eq!(url.urn().to_str()?, case.urn);
			assert_eq!(url.uri().to_str()?, case.uri);
			assert_eq!(format!("{}", url.trail()), format!("{}", UrlCow::try_from(case.trail)?));
			assert_eq!(format!("{}", url.base()), format!("{}", UrlCow::try_from(case.base)?));
		}

		Ok(())
	}
}
