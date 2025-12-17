use std::{borrow::Cow, hash::{Hash, Hasher}, path::PathBuf};

use anyhow::{Result, anyhow};
use serde::{Deserialize, Deserializer, Serialize};

use crate::{loc::{Loc, LocBuf}, path::{PathBufDyn, PathCow, PathDyn}, pool::SymbolCow, scheme::{AsScheme, Scheme, SchemeCow, SchemeKind, SchemeRef}, url::{AsUrl, Url, UrlBuf}};

#[derive(Clone, Debug)]
pub enum UrlCow<'a> {
	Regular(LocBuf),
	Search { loc: LocBuf, domain: SymbolCow<'a, str> },
	Archive { loc: LocBuf, domain: SymbolCow<'a, str> },
	Sftp { loc: LocBuf<typed_path::UnixPathBuf>, domain: SymbolCow<'a, str> },

	RegularRef(Loc<'a>),
	SearchRef { loc: Loc<'a>, domain: SymbolCow<'a, str> },
	ArchiveRef { loc: Loc<'a>, domain: SymbolCow<'a, str> },
	SftpRef { loc: Loc<'a, &'a typed_path::UnixPath>, domain: SymbolCow<'a, str> },
}

// FIXME: remove
impl Default for UrlCow<'_> {
	fn default() -> Self { Self::RegularRef(Default::default()) }
}

impl<'a> From<Url<'a>> for UrlCow<'a> {
	fn from(value: Url<'a>) -> Self {
		match value {
			Url::Regular(loc) => Self::RegularRef(loc),
			Url::Search { loc, domain } => Self::SearchRef { loc, domain: domain.into() },
			Url::Archive { loc, domain } => Self::ArchiveRef { loc, domain: domain.into() },
			Url::Sftp { loc, domain } => Self::SftpRef { loc, domain: domain.into() },
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
			UrlBuf::Regular(loc) => Self::Regular(loc),
			UrlBuf::Search { loc, domain } => Self::Search { loc, domain: domain.into() },
			UrlBuf::Archive { loc, domain } => Self::Archive { loc, domain: domain.into() },
			UrlBuf::Sftp { loc, domain } => Self::Sftp { loc, domain: domain.into() },
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

	fn try_from(value: &'a [u8]) -> Result<Self, Self::Error> { SchemeCow::parse(value)?.try_into() }
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

impl<'a> TryFrom<(SchemeRef<'a>, PathDyn<'a>)> for UrlCow<'a> {
	type Error = anyhow::Error;

	fn try_from((scheme, path): (SchemeRef<'a>, PathDyn<'a>)) -> Result<Self, Self::Error> {
		(SchemeCow::Borrowed(scheme), path).try_into()
	}
}

impl<'a> TryFrom<(SchemeRef<'a>, PathCow<'a>)> for UrlCow<'a> {
	type Error = anyhow::Error;

	fn try_from((scheme, path): (SchemeRef<'a>, PathCow<'a>)) -> Result<Self, Self::Error> {
		(SchemeCow::Borrowed(scheme), path).try_into()
	}
}

impl TryFrom<(Scheme, PathBufDyn)> for UrlCow<'_> {
	type Error = anyhow::Error;

	fn try_from((scheme, path): (Scheme, PathBufDyn)) -> Result<Self, Self::Error> {
		(SchemeCow::Owned(scheme), path).try_into()
	}
}

impl<'a> TryFrom<(SchemeCow<'a>, PathCow<'a>)> for UrlCow<'a> {
	type Error = anyhow::Error;

	fn try_from((scheme, path): (SchemeCow<'a>, PathCow<'a>)) -> Result<Self, Self::Error> {
		match path {
			PathCow::Borrowed(path) => (scheme, path).try_into(),
			PathCow::Owned(path) => (scheme, path).try_into(),
		}
	}
}

impl<'a> TryFrom<(SchemeCow<'a>, PathDyn<'a>)> for UrlCow<'a> {
	type Error = anyhow::Error;

	fn try_from((scheme, path): (SchemeCow<'a>, PathDyn<'a>)) -> Result<Self, Self::Error> {
		let kind = scheme.as_scheme().kind();
		let (uri, urn) = scheme.as_scheme().ports();
		let domain = scheme.into_domain();
		Ok(match kind {
			SchemeKind::Regular => Self::RegularRef(Loc::bare(path.as_os()?)),
			SchemeKind::Search => Self::SearchRef {
				loc:    Loc::with(path.as_os()?, uri, urn)?,
				domain: domain.ok_or_else(|| anyhow!("missing domain for search scheme"))?,
			},
			SchemeKind::Archive => Self::ArchiveRef {
				loc:    Loc::with(path.as_os()?, uri, urn)?,
				domain: domain.ok_or_else(|| anyhow!("missing domain for archive scheme"))?,
			},
			SchemeKind::Sftp => Self::SftpRef {
				loc:    Loc::with(path.as_unix()?, uri, urn)?,
				domain: domain.ok_or_else(|| anyhow!("missing domain for sftp scheme"))?,
			},
		})
	}
}

impl<'a> TryFrom<(SchemeCow<'a>, PathBufDyn)> for UrlCow<'a> {
	type Error = anyhow::Error;

	fn try_from((scheme, path): (SchemeCow<'a>, PathBufDyn)) -> Result<Self, Self::Error> {
		let kind = scheme.as_scheme().kind();
		let (uri, urn) = scheme.as_scheme().ports();
		let domain = scheme.into_domain();
		Ok(match kind {
			SchemeKind::Regular => Self::Regular(path.into_os()?.into()),
			SchemeKind::Search => Self::Search {
				loc:    LocBuf::<std::path::PathBuf>::with(path.try_into()?, uri, urn)?,
				domain: domain.ok_or_else(|| anyhow!("missing domain for search scheme"))?,
			},
			SchemeKind::Archive => Self::Archive {
				loc:    LocBuf::<std::path::PathBuf>::with(path.try_into()?, uri, urn)?,
				domain: domain.ok_or_else(|| anyhow!("missing domain for archive scheme"))?,
			},
			SchemeKind::Sftp => Self::Sftp {
				loc:    LocBuf::<typed_path::UnixPathBuf>::with(path.try_into()?, uri, urn)?,
				domain: domain.ok_or_else(|| anyhow!("missing domain for sftp scheme"))?,
			},
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
			Self::Regular(_) | Self::Search { .. } | Self::Archive { .. } | Self::Sftp { .. } => true,
			Self::RegularRef(_)
			| Self::SearchRef { .. }
			| Self::ArchiveRef { .. }
			| Self::SftpRef { .. } => false,
		}
	}

	pub fn into_owned(self) -> UrlBuf {
		match self {
			Self::Regular(loc) => UrlBuf::Regular(loc),
			Self::Search { loc, domain } => UrlBuf::Search { loc, domain: domain.into() },
			Self::Archive { loc, domain } => UrlBuf::Archive { loc, domain: domain.into() },
			Self::Sftp { loc, domain } => UrlBuf::Sftp { loc, domain: domain.into() },

			Self::RegularRef(loc) => UrlBuf::Regular(loc.into()),
			Self::SearchRef { loc, domain } => {
				UrlBuf::Search { loc: loc.into(), domain: domain.into() }
			}
			Self::ArchiveRef { loc, domain } => {
				UrlBuf::Archive { loc: loc.into(), domain: domain.into() }
			}
			Self::SftpRef { loc, domain } => UrlBuf::Sftp { loc: loc.into(), domain: domain.into() },
		}
	}

	pub fn into_scheme(self) -> SchemeCow<'a> {
		let (uri, urn) = self.as_url().scheme().ports();
		match self {
			Self::Regular(_) => Scheme::Regular { uri, urn }.into(),
			Self::RegularRef(_) => SchemeRef::Regular { uri, urn }.into(),
			Self::Search { domain, .. } | Self::SearchRef { domain, .. } => match domain {
				SymbolCow::Borrowed(domain) => SchemeRef::Search { domain, uri, urn }.into(),
				SymbolCow::Owned(domain) => Scheme::Search { domain, uri, urn }.into(),
			},
			Self::Archive { domain, .. } | Self::ArchiveRef { domain, .. } => match domain {
				SymbolCow::Borrowed(domain) => SchemeRef::Archive { domain, uri, urn }.into(),
				SymbolCow::Owned(domain) => Scheme::Archive { domain, uri, urn }.into(),
			},
			Self::Sftp { domain, .. } | Self::SftpRef { domain, .. } => match domain {
				SymbolCow::Borrowed(domain) => SchemeRef::Sftp { domain, uri, urn }.into(),
				SymbolCow::Owned(domain) => Scheme::Sftp { domain, uri, urn }.into(),
			},
		}
	}

	pub fn into_static(self) -> UrlCow<'static> {
		match self {
			UrlCow::Regular(loc) => UrlCow::Regular(loc),
			UrlCow::Search { loc, domain } => UrlCow::Search { loc, domain: domain.into_owned().into() },
			UrlCow::Archive { loc, domain } => {
				UrlCow::Archive { loc, domain: domain.into_owned().into() }
			}
			UrlCow::Sftp { loc, domain } => UrlCow::Sftp { loc, domain: domain.into_owned().into() },

			UrlCow::RegularRef(loc) => UrlCow::Regular(loc.into()),
			UrlCow::SearchRef { loc, domain } => {
				UrlCow::Search { loc: loc.into(), domain: domain.into_owned().into() }
			}
			UrlCow::ArchiveRef { loc, domain } => {
				UrlCow::Archive { loc: loc.into(), domain: domain.into_owned().into() }
			}
			UrlCow::SftpRef { loc, domain } => {
				UrlCow::Sftp { loc: loc.into(), domain: domain.into_owned().into() }
			}
		}
	}

	#[inline]
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

#[cfg(test)]
mod tests {
	use super::*;
	use crate::url::UrlLike;

	#[test]
	fn test_parse() -> Result<()> {
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
			// Archive portal
			Case {
				url:   "archive://domain//root/Downloads/images.zip",
				urn:   "",
				uri:   "",
				trail: "archive://domain//root/Downloads/images.zip",
				base:  "archive://domain//root/Downloads/images.zip",
			},
			// Archive item
			Case {
				url:   "archive://domain:2:1//root/Downloads/images.zip/2025/city.jpg",
				urn:   "city.jpg",
				uri:   "2025/city.jpg",
				trail: "archive://domain:1:1//root/Downloads/images.zip/2025/",
				base:  "archive://domain//root/Downloads/images.zip/",
			},
			// SFTP
			Case {
				url:   "sftp://my-server//root/docs/report.pdf",
				urn:   "report.pdf",
				uri:   "report.pdf",
				trail: "sftp://my-server//root/docs/",
				base:  "sftp://my-server//root/docs/",
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
