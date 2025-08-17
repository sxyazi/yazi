use std::{borrow::Cow, path::{Path, PathBuf}};

use anyhow::Result;
use percent_encoding::percent_decode;

use crate::{IntoOsStr, loc::{Loc, LocBuf}, url::{Components, Scheme, Url, UrlBuf, UrnBuf}};

#[derive(Debug)]
pub enum UrlCow<'a> {
	Borrowed(Url<'a>),
	Owned(UrlBuf),
}

impl Default for UrlCow<'_> {
	fn default() -> Self { Self::Owned(UrlBuf::default()) }
}

impl<'a> From<Url<'a>> for UrlCow<'a> {
	fn from(value: Url<'a>) -> Self { Self::Borrowed(value) }
}

impl<'a> From<&'a UrlBuf> for UrlCow<'a> {
	fn from(value: &'a UrlBuf) -> Self { Self::Borrowed(value.into()) }
}

impl From<UrlBuf> for UrlCow<'_> {
	fn from(value: UrlBuf) -> Self { Self::Owned(value) }
}

impl<'a> From<&'a UrlCow<'a>> for Url<'a> {
	fn from(value: &'a UrlCow<'a>) -> Self { value.as_url() }
}

impl From<UrlCow<'_>> for UrlBuf {
	fn from(value: UrlCow<'_>) -> Self { value.into_owned() }
}

impl<'a> TryFrom<&'a [u8]> for UrlCow<'a> {
	type Error = anyhow::Error;

	fn try_from(value: &'a [u8]) -> Result<Self, Self::Error> {
		let (scheme, path, port) = Self::parse(value)?;

		Ok(match (path, port) {
			(Cow::Borrowed(p), None) => Url { loc: Loc::from(p), scheme }.into(),
			(Cow::Borrowed(p), Some((uri, urn))) => Url { loc: Loc::with(p, uri, urn)?, scheme }.into(),
			(Cow::Owned(p), None) => UrlBuf { loc: LocBuf::from(p), scheme }.into(),
			(Cow::Owned(p), Some((uri, urn))) => {
				UrlBuf { loc: LocBuf::with(p, uri, urn)?, scheme }.into()
			}
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

impl UrlCow<'_> {
	#[inline]
	pub fn loc(&self) -> Loc<'_> {
		match self {
			UrlCow::Borrowed(u) => u.loc.as_loc(),
			UrlCow::Owned(u) => u.loc.as_loc(),
		}
	}

	#[inline]
	pub fn scheme(&self) -> &Scheme {
		match self {
			UrlCow::Borrowed(u) => &u.scheme,
			UrlCow::Owned(u) => &u.scheme,
		}
	}

	pub fn as_url(&self) -> Url<'_> {
		match self {
			UrlCow::Borrowed(u) => u.as_url(),
			UrlCow::Owned(u) => u.as_url(),
		}
	}

	#[inline]
	pub fn into_owned(self) -> UrlBuf { self.as_url().into() }

	#[inline]
	pub fn parent_url(&self) -> Option<UrlBuf> { self.as_url().parent_url() }

	#[inline]
	pub fn pair(&self) -> Option<(UrlBuf, UrnBuf)> { self.as_url().pair() }

	pub fn parse(bytes: &[u8]) -> Result<(Scheme, Cow<'_, Path>, Option<(usize, usize)>)> {
		let mut skip = 0;
		let (scheme, tilde, uri, urn) = Scheme::parse(bytes, &mut skip)?;

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
