use std::{borrow::Cow, path::{Path, PathBuf}};

use anyhow::Result;
use percent_encoding::percent_decode;

use crate::{IntoOsStr, loc::{Loc, LocBuf}, scheme::{SchemeCow, SchemeRef}, url::{Components, Url, UrlBuf, Urn}};

#[derive(Debug)]
pub enum UrlCow<'a> {
	Borrowed { loc: Loc<'a>, scheme: SchemeCow<'a> },
	Owned { loc: LocBuf, scheme: SchemeCow<'a> },
}

impl Default for UrlCow<'_> {
	fn default() -> Self { Self::Owned { loc: Default::default(), scheme: Default::default() } }
}

impl<'a> From<Url<'a>> for UrlCow<'a> {
	fn from(value: Url<'a>) -> Self { Self::Borrowed { loc: value.loc, scheme: value.scheme.into() } }
}

impl<'a> From<&'a UrlBuf> for UrlCow<'a> {
	fn from(value: &'a UrlBuf) -> Self {
		Self::Borrowed { loc: value.loc.as_loc(), scheme: SchemeCow::from(&value.scheme) }
	}
}

impl From<UrlBuf> for UrlCow<'_> {
	fn from(value: UrlBuf) -> Self { Self::Owned { loc: value.loc, scheme: value.scheme.into() } }
}

impl<'a> From<&'a UrlCow<'a>> for Url<'a> {
	fn from(value: &'a UrlCow<'a>) -> Self { value.as_url() }
}

impl From<UrlCow<'_>> for UrlBuf {
	fn from(value: UrlCow<'_>) -> Self { value.into_owned() }
}

impl From<&UrlCow<'_>> for UrlBuf {
	fn from(value: &UrlCow<'_>) -> Self { value.as_url().into() }
}

impl<'a> TryFrom<&'a [u8]> for UrlCow<'a> {
	type Error = anyhow::Error;

	fn try_from(value: &'a [u8]) -> Result<Self, Self::Error> {
		let (scheme, path, port) = Self::parse(value)?;

		Ok(match (path, port) {
			(Cow::Borrowed(p), None) => Self::Borrowed { loc: Loc::from(p), scheme },
			(Cow::Borrowed(p), Some((uri, urn))) => {
				Self::Borrowed { loc: Loc::with(p, uri, urn)?, scheme }
			}
			(Cow::Owned(p), None) => Self::Owned { loc: LocBuf::from(p), scheme },
			(Cow::Owned(p), Some((uri, urn))) => Self::Owned { loc: LocBuf::with(p, uri, urn)?, scheme },
		})
	}
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

// --- Eq
impl PartialEq<UrlBuf> for UrlCow<'_> {
	fn eq(&self, other: &UrlBuf) -> bool { self.as_url() == other.as_url() }
}

impl<'a> UrlCow<'a> {
	#[inline]
	pub fn loc(&self) -> Loc<'_> {
		match self {
			UrlCow::Borrowed { loc, .. } => *loc,
			UrlCow::Owned { loc, .. } => loc.as_loc(),
		}
	}

	#[inline]
	pub fn scheme(&self) -> SchemeRef<'_> {
		match self {
			UrlCow::Borrowed { scheme, .. } => scheme.as_ref(),
			UrlCow::Owned { scheme, .. } => scheme.as_ref(),
		}
	}

	#[inline]
	pub fn as_url(&self) -> Url<'_> {
		match self {
			UrlCow::Borrowed { loc, scheme } => Url { loc: *loc, scheme: scheme.as_ref() },
			UrlCow::Owned { loc, scheme } => Url { loc: loc.as_loc(), scheme: scheme.as_ref() },
		}
	}

	#[inline]
	pub fn into_owned(self) -> UrlBuf {
		match self {
			UrlCow::Borrowed { loc, scheme } => UrlBuf { loc: loc.into(), scheme: scheme.into() },
			UrlCow::Owned { loc, scheme } => UrlBuf { loc, scheme: scheme.into() },
		}
	}

	#[inline]
	pub fn into_scheme(self) -> SchemeCow<'a> {
		match self {
			UrlCow::Borrowed { scheme, .. } => scheme,
			UrlCow::Owned { scheme, .. } => scheme,
		}
	}

	#[inline]
	pub fn parent(&self) -> Option<Url<'_>> { self.as_url().parent() }

	#[inline]
	pub fn pair(&self) -> Option<(Url<'_>, &Urn)> { self.as_url().pair() }

	pub fn parse(bytes: &[u8]) -> Result<(SchemeCow<'_>, Cow<'_, Path>, Option<(usize, usize)>)> {
		let mut skip = 0;
		let (scheme, tilde, uri, urn) = SchemeCow::parse(bytes, &mut skip)?;

		let rest = if tilde {
			Cow::from(percent_decode(&bytes[skip..])).into_os_str()?
		} else {
			bytes[skip..].into_os_str()?
		};

		let path: Cow<_> = match rest {
			Cow::Borrowed(s) => Path::new(s).into(),
			Cow::Owned(s) => PathBuf::from(s).into(),
		};

		let ports = scheme.normalize_ports(uri, urn, &path)?;

		Ok((scheme, path, ports))
	}
}

impl UrlCow<'_> {
	#[inline]
	pub fn is_regular(&self) -> bool { self.as_url().is_regular() }

	#[inline]
	pub fn is_absolute(&self) -> bool { self.as_url().is_absolute() }

	#[inline]
	pub fn components(&self) -> Components<'_> { Components::from(self) }

	#[inline]
	pub fn covariant(&self, other: &Self) -> bool { self.as_url().covariant(other) }
}
