use std::{borrow::Cow, hash::{Hash, Hasher}, path::PathBuf, sync::Arc};

use anyhow::{Result, ensure};
use serde::{Deserialize, Deserializer, Serialize};
use typed_path::{UnixPath, UnixPathBuf};

use crate::{auth::{Auth, AuthKind}, loc::{Loc, LocBuf, LocCow}, path::{DynPath, PathBufDyn, PathCow, PathDyn}, spec::Spec, url::{AsUrl, Url, UrlBuf}};

#[derive(Clone, Debug)]
pub enum UrlCow<'a> {
	Regular(LocCow<'a>),
	Search { loc: LocCow<'a>, auth: Arc<Auth> },
	Mount { loc: LocCow<'a>, auth: Arc<Auth> },
	Hub { loc: LocCow<'a>, auth: Arc<Auth> },
	Scope { loc: LocCow<'a, &'a UnixPath, UnixPathBuf>, auth: Arc<Auth> },
	Sftp { loc: LocCow<'a, &'a UnixPath, UnixPathBuf>, auth: Arc<Auth> },
}

impl<'a> From<Url<'a>> for UrlCow<'a> {
	fn from(value: Url<'a>) -> Self {
		match value {
			Url::Regular(loc) => Self::Regular(loc.into()),
			Url::Search { loc, auth } => Self::Search { loc: loc.into(), auth: auth.clone() },
			Url::Mount { loc, auth } => Self::Mount { loc: loc.into(), auth: auth.clone() },
			Url::Hub { loc, auth } => Self::Hub { loc: loc.into(), auth: auth.clone() },
			Url::Scope { loc, auth } => Self::Scope { loc: loc.into(), auth: auth.clone() },
			Url::Sftp { loc, auth } => Self::Sftp { loc: loc.into(), auth: auth.clone() },
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
			UrlBuf::Regular(loc) => Self::Regular(loc.into()),
			UrlBuf::Search { loc, auth } => Self::Search { loc: loc.into(), auth },
			UrlBuf::Mount { loc, auth } => Self::Mount { loc: loc.into(), auth },
			UrlBuf::Hub { loc, auth } => Self::Hub { loc: loc.into(), auth },
			UrlBuf::Scope { loc, auth } => Self::Scope { loc: loc.into(), auth },
			UrlBuf::Sftp { loc, auth } => Self::Sftp { loc: loc.into(), auth },
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
		validate_auth_depth(&auth, path)?;

		Ok(match auth.kind {
			AuthKind::Regular => Self::Regular(Loc::bare(path.as_os()?).into()),
			AuthKind::Search => Self::Search { loc: Loc::with(path.as_os()?, uri, urn)?.into(), auth },
			AuthKind::Mount => Self::Mount { loc: Loc::with(path.as_os()?, uri, urn)?.into(), auth },
			AuthKind::Hub => Self::Hub { loc: Loc::with(path.as_os()?, uri, urn)?.into(), auth },
			AuthKind::Scope => Self::Scope { loc: Loc::with(path.as_unix()?, uri, urn)?.into(), auth },
			AuthKind::Sftp => Self::Sftp { loc: Loc::with(path.as_unix()?, uri, urn)?.into(), auth },
		})
	}
}

impl<'a> TryFrom<(Spec, PathBufDyn)> for UrlCow<'a> {
	type Error = anyhow::Error;

	fn try_from((spec, path): (Spec, PathBufDyn)) -> Result<Self, Self::Error> {
		let Spec { auth, uri, urn } = spec;
		validate_auth_depth(&auth, path.dyn_path())?;

		Ok(match auth.kind {
			AuthKind::Regular => {
				Self::Regular(LocBuf::<std::path::PathBuf>::from(path.into_os()?).into())
			}
			AuthKind::Search => Self::Search {
				loc: LocBuf::<std::path::PathBuf>::with(path.try_into()?, uri, urn)?.into(),
				auth,
			},
			AuthKind::Mount => Self::Mount {
				loc: LocBuf::<std::path::PathBuf>::with(path.try_into()?, uri, urn)?.into(),
				auth,
			},
			AuthKind::Hub => {
				Self::Hub { loc: LocBuf::<PathBuf>::with(path.try_into()?, uri, urn)?.into(), auth }
			}
			AuthKind::Scope => {
				Self::Scope { loc: LocBuf::<UnixPathBuf>::with(path.try_into()?, uri, urn)?.into(), auth }
			}
			AuthKind::Sftp => {
				Self::Sftp { loc: LocBuf::<UnixPathBuf>::with(path.try_into()?, uri, urn)?.into(), auth }
			}
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
			Self::Regular(loc) => loc.is_owned(),
			Self::Search { loc, .. } => loc.is_owned(),
			Self::Mount { loc, .. } => loc.is_owned(),
			Self::Hub { loc, .. } => loc.is_owned(),
			Self::Scope { loc, .. } => loc.is_owned(),
			Self::Sftp { loc, .. } => loc.is_owned(),
		}
	}

	pub fn into_owned(self) -> UrlBuf {
		match self {
			Self::Regular(loc) => UrlBuf::Regular(loc.into_owned()),
			Self::Search { loc, auth } => UrlBuf::Search { loc: loc.into_owned(), auth },
			Self::Mount { loc, auth } => UrlBuf::Mount { loc: loc.into_owned(), auth },
			Self::Hub { loc, auth } => UrlBuf::Hub { loc: loc.into_owned(), auth },
			Self::Scope { loc, auth } => UrlBuf::Scope { loc: loc.into_owned(), auth },
			Self::Sftp { loc, auth } => UrlBuf::Sftp { loc: loc.into_owned(), auth },
		}
	}

	pub fn into_pair(self) -> (Spec, PathCow<'a>) {
		let (uri, urn) = Spec::retrieve_ports(self.as_url());
		let (auth, path) = match self {
			Self::Regular(loc) => (Auth::default_arc(), loc.into_path()),
			Self::Search { loc, auth } | Self::Mount { loc, auth } | Self::Hub { loc, auth } => {
				(auth, loc.into_path())
			}
			Self::Scope { loc, auth } | Self::Sftp { loc, auth } => (auth, loc.into_path()),
		};
		(Spec { auth, uri, urn }, path)
	}

	pub fn into_spec(self) -> Spec { self.into_pair().0 }

	pub fn into_path(self) -> PathCow<'a> { self.into_pair().1 }

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

fn validate_auth_depth(auth: &Auth, path: PathDyn) -> Result<()> {
	if auth.kind == AuthKind::Hub {
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
			// Search portal
			Case {
				url:   "search://keyword//root/Documents/reports",
				urn:   "",
				uri:   "",
				trail: "search://keyword//root/Documents/reports",
				base:  "search://keyword//root/Documents/reports",
			},
			// Search item
			Case {
				url:   "search://keyword:2:2//root/Documents/reports/2023/summary.docx",
				urn:   "2023/summary.docx",
				uri:   "2023/summary.docx",
				trail: "search://keyword//root/Documents/reports/",
				base:  "search://keyword//root/Documents/reports/",
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
			assert_eq!(
				format!("{:?}", url.trail()),
				format!("{:?}", UrlCow::try_from(case.trail)?.as_url())
			);
			assert_eq!(
				format!("{:?}", url.base()),
				format!("{:?}", UrlCow::try_from(case.base)?.as_url())
			);
		}

		Ok(())
	}
}
