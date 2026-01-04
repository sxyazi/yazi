use std::borrow::Cow;

use anyhow::{Result, ensure};
use percent_encoding::percent_decode;

use crate::{path::{PathCow, PathLike}, pool::{InternStr, SymbolCow}, scheme::{AsScheme, Scheme, SchemeKind, SchemeRef}, url::Url};

#[derive(Clone, Debug)]
pub enum SchemeCow<'a> {
	Borrowed(SchemeRef<'a>),
	Owned(Scheme),
}

impl<'a> From<SchemeRef<'a>> for SchemeCow<'a> {
	fn from(value: SchemeRef<'a>) -> Self { Self::Borrowed(value) }
}

impl<'a, T> From<&'a T> for SchemeCow<'a>
where
	T: AsScheme + ?Sized,
{
	fn from(value: &'a T) -> Self { Self::Borrowed(value.as_scheme()) }
}

impl From<Scheme> for SchemeCow<'_> {
	fn from(value: Scheme) -> Self { Self::Owned(value) }
}

impl From<SchemeCow<'_>> for Scheme {
	fn from(value: SchemeCow<'_>) -> Self { value.into_owned() }
}

impl PartialEq<SchemeRef<'_>> for SchemeCow<'_> {
	fn eq(&self, other: &SchemeRef) -> bool { self.as_scheme() == *other }
}

impl<'a> SchemeCow<'a> {
	pub fn regular(uri: usize, urn: usize) -> Self { SchemeRef::Regular { uri, urn }.into() }

	pub fn search<T>(domain: T, uri: usize, urn: usize) -> Self
	where
		T: Into<Cow<'a, str>>,
	{
		match domain.into() {
			Cow::Borrowed(domain) => SchemeRef::Search { domain, uri, urn }.into(),
			Cow::Owned(domain) => Scheme::Search { domain: domain.intern(), uri, urn }.into(),
		}
	}

	pub fn archive<T>(domain: T, uri: usize, urn: usize) -> Self
	where
		T: Into<Cow<'a, str>>,
	{
		match domain.into() {
			Cow::Borrowed(domain) => SchemeRef::Archive { domain, uri, urn }.into(),
			Cow::Owned(domain) => Scheme::Archive { domain: domain.intern(), uri, urn }.into(),
		}
	}

	pub fn sftp<T>(domain: T, uri: usize, urn: usize) -> Self
	where
		T: Into<Cow<'a, str>>,
	{
		match domain.into() {
			Cow::Borrowed(domain) => SchemeRef::Sftp { domain, uri, urn }.into(),
			Cow::Owned(domain) => Scheme::Sftp { domain: domain.intern(), uri, urn }.into(),
		}
	}

	pub fn parse(bytes: &'a [u8]) -> Result<(Self, PathCow<'a>)> {
		let Some((kind, tilde)) = SchemeKind::parse(bytes)? else {
			let path = Self::decode_path(SchemeKind::Regular, false, bytes)?;
			let (uri, urn) = Self::normalize_ports(SchemeKind::Regular, None, None, &path)?;
			return Ok((Self::regular(uri, urn), path));
		};

		// Decode domain and ports
		let mut skip = kind.offset(tilde);
		let (domain, uri, urn) = match kind {
			SchemeKind::Regular => ("".into(), None, None),
			SchemeKind::Search => Self::decode_param(&bytes[skip..], &mut skip)?,
			SchemeKind::Archive => Self::decode_param(&bytes[skip..], &mut skip)?,
			SchemeKind::Sftp => Self::decode_param(&bytes[skip..], &mut skip)?,
		};

		// Decode path
		let path = Self::decode_path(kind, tilde, &bytes[skip..])?;

		// Build scheme
		let (uri, urn) = Self::normalize_ports(kind, uri, urn, &path)?;
		let scheme = match kind {
			SchemeKind::Regular => Self::regular(uri, urn),
			SchemeKind::Search => Self::search(domain, uri, urn),
			SchemeKind::Archive => Self::archive(domain, uri, urn),
			SchemeKind::Sftp => Self::sftp(domain, uri, urn),
		};

		Ok((scheme, path))
	}

	fn decode_param(
		bytes: &'a [u8],
		skip: &mut usize,
	) -> Result<(Cow<'a, str>, Option<usize>, Option<usize>)> {
		let mut len = bytes.iter().copied().take_while(|&b| b != b'/').count();
		let slash = bytes.get(len).is_some_and(|&b| b == b'/');
		*skip += len + slash as usize;

		let (uri, urn) = Self::decode_ports(&bytes[..len], &mut len)?;
		let domain = match Cow::from(percent_decode(&bytes[..len])) {
			Cow::Borrowed(b) => str::from_utf8(b)?.into(),
			Cow::Owned(b) => String::from_utf8(b)?.into(),
		};

		Ok((domain, uri, urn))
	}

	fn decode_ports(bytes: &[u8], skip: &mut usize) -> Result<(Option<usize>, Option<usize>)> {
		let Some(a_idx) = bytes.iter().rposition(|&b| b == b':') else { return Ok((None, None)) };
		let a_len = bytes.len() - a_idx;
		*skip -= a_len;
		let a = if a_len == 1 { None } else { Some(str::from_utf8(&bytes[a_idx + 1..])?.parse()?) };

		let Some(b_idx) = bytes[..a_idx].iter().rposition(|&b| b == b':') else {
			return Ok((a, None));
		};
		let b_len = bytes[..a_idx].len() - b_idx;
		*skip -= b_len;
		let b =
			if b_len == 1 { None } else { Some(str::from_utf8(&bytes[b_idx + 1..a_idx])?.parse()?) };

		Ok((b, a))
	}

	fn decode_path(kind: SchemeKind, tilde: bool, bytes: &'a [u8]) -> Result<PathCow<'a>> {
		let bytes: Cow<_> = if tilde { percent_decode(bytes).into() } else { bytes.into() };
		PathCow::with(kind, bytes)
	}

	fn normalize_ports(
		kind: SchemeKind,
		uri: Option<usize>,
		urn: Option<usize>,
		path: &PathCow,
	) -> Result<(usize, usize)> {
		Ok(match kind {
			SchemeKind::Regular => {
				ensure!(uri.is_none() && urn.is_none(), "Regular scheme cannot have ports");
				(path.name().is_some() as usize, path.name().is_some() as usize)
			}
			SchemeKind::Search => {
				let (uri, urn) = (uri.unwrap_or(0), urn.unwrap_or(0));
				ensure!(uri == urn, "Search scheme requires URI and URN to be equal");
				(uri, urn)
			}
			SchemeKind::Archive => (uri.unwrap_or(0), urn.unwrap_or(0)),
			SchemeKind::Sftp => {
				let uri = uri.unwrap_or(path.name().is_some() as usize);
				let urn = urn.unwrap_or(path.name().is_some() as usize);
				(uri, urn)
			}
		})
	}

	pub fn retrieve_ports(url: Url) -> (usize, usize) {
		match url {
			Url::Regular(loc) => (loc.file_name().is_some() as usize, loc.file_name().is_some() as usize),
			Url::Search { loc, .. } => (loc.uri().components().count(), loc.urn().components().count()),
			Url::Archive { loc, .. } => (loc.uri().components().count(), loc.urn().components().count()),
			Url::Sftp { loc, .. } => (loc.uri().components().count(), loc.urn().components().count()),
		}
	}
}

impl<'a> SchemeCow<'a> {
	#[inline]
	pub fn into_domain(self) -> Option<SymbolCow<'a, str>> {
		Some(match self {
			SchemeCow::Borrowed(s) => s.domain()?.into(),
			SchemeCow::Owned(s) => s.into_domain()?.into(),
		})
	}

	#[inline]
	pub fn into_owned(self) -> Scheme {
		match self {
			Self::Borrowed(s) => s.to_owned(),
			Self::Owned(s) => s,
		}
	}

	pub fn with_ports(self, uri: usize, urn: usize) -> Self {
		match self {
			Self::Borrowed(s) => s.with_ports(uri, urn).into(),
			Self::Owned(s) => s.with_ports(uri, urn).into(),
		}
	}

	#[inline]
	pub fn zeroed(self) -> Self { self.with_ports(0, 0) }
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_decode_ports() -> Result<()> {
		fn assert(s: &str, len: usize, uri: Option<usize>, urn: Option<usize>) -> Result<()> {
			let mut n = usize::MAX;
			let port = SchemeCow::decode_ports(s.as_bytes(), &mut n)?;
			assert_eq!((usize::MAX - n, port.0, port.1), (len, uri, urn));
			Ok(())
		}

		// Zeros
		assert("", 0, None, None)?;
		assert(":", 1, None, None)?;
		assert("::", 2, None, None)?;

		// URI
		assert(":2", 2, Some(2), None)?;
		assert(":2:", 3, Some(2), None)?;
		assert(":22:", 4, Some(22), None)?;

		// URN
		assert("::1", 3, None, Some(1))?;
		assert(":2:1", 4, Some(2), Some(1))?;
		assert(":22:11", 6, Some(22), Some(11))?;
		Ok(())
	}
}
