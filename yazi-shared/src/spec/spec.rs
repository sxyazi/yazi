use std::{borrow::Cow, ops::Deref, sync::Arc};

use anyhow::{Result, anyhow, ensure};
use percent_encoding::percent_decode;
use serde::Deserialize;

use crate::{auth::{Auth, AuthKind, Domain, EncodeAuth, Scheme}, path::{PathCow, PathLike}, spec::ParsedSpec, url::Url};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct Spec {
	#[serde(flatten)]
	pub auth: Arc<Auth>,
	pub uri:  usize,
	pub urn:  usize,
}

impl Deref for Spec {
	type Target = Auth;

	fn deref(&self) -> &Self::Target { &self.auth }
}

impl Spec {
	pub fn parse<'a>(bytes: &'a [u8]) -> Result<(Self, PathCow<'a>)> {
		let parsed = ParsedSpec::parse(bytes)?;
		let rest = parsed.rest();

		// Decode domain and ports
		let mut skip = 0;
		let (domain, uri, urn) = match &parsed.scheme {
			Scheme::Regular => (Domain::default(), None, None),
			_ => Self::decode_param(rest, &mut skip)?,
		};

		// Resolve authority
		let auth = Auth::get(&parsed.scheme, &domain).ok_or_else(|| {
			anyhow!("unknown VFS authority: {parsed}://{}", EncodeAuth::domain(&domain))
		})?;

		// Decode path
		let (path, auth) = if auth.kind == AuthKind::Hub {
			Self::decode_hub(auth, parsed.tilde, &rest[skip..])?
		} else {
			(Self::decode_path(auth.kind, parsed.tilde, &rest[skip..])?, auth)
		};

		let (uri, urn) = Self::normalize_ports(auth.kind, uri, urn, &path)?;
		Ok((Self { auth, uri, urn }, path))
	}

	pub const fn ports(&self) -> (usize, usize) { (self.uri, self.urn) }

	#[inline]
	pub fn with_ports(self, uri: usize, urn: usize) -> Self { Self { uri, urn, ..self } }

	#[inline]
	pub fn zeroed(self) -> Self { self.with_ports(0, 0) }

	fn decode_param<'a>(
		bytes: &'a [u8],
		skip: &mut usize,
	) -> Result<(Domain<'a>, Option<usize>, Option<usize>)> {
		let mut len = bytes.iter().copied().take_while(|&b| b != b'/').count();
		let slash = bytes.get(len).is_some_and(|&b| b == b'/');
		*skip += len + slash as usize;

		let (uri, urn) = Self::decode_ports(&bytes[..len], &mut len)?;
		let domain: Cow<[u8]> = percent_decode(&bytes[..len]).into();

		Ok((domain.into(), uri, urn))
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

	fn decode_path<'a>(kind: AuthKind, tilde: bool, bytes: &'a [u8]) -> Result<PathCow<'a>> {
		let bytes: Cow<_> = if tilde { percent_decode(bytes).into() } else { bytes.into() };
		PathCow::with(kind, bytes)
	}

	fn decode_hub<'a>(
		mut auth: Arc<Auth>,
		tilde: bool,
		bytes: &'a [u8],
	) -> Result<(PathCow<'a>, Arc<Auth>)> {
		ensure!(bytes.first() == Some(&b'@'), "Hub URL requires an `@` parent marker");
		let end = bytes[1..]
			.iter()
			.position(|&b| b == b'/')
			.map(|i| i + 1)
			.ok_or_else(|| anyhow!("Hub URL requires a path delimiter"))?;

		let path = Self::decode_path(AuthKind::Hub, tilde, &bytes[end + 1..])?;
		let depth = path.components().auth_depth();
		if depth == 0 {
			ensure!(bytes[1..end].is_empty(), "Hub URL has too many parent domains");
			return Ok((path, auth));
		}

		let (mut count, mut parent) = (0, None);
		for domain in bytes[1..end].rsplit(|&b| b == b',') {
			count += 1;
			parent = Some(Arc::new(Auth {
				kind: auth.kind,
				scheme: auth.scheme.clone(),
				domain: Domain::from(Cow::from(percent_decode(domain))).into_owned(),
				parent,
			}));
		}

		ensure!(count == depth, "Hub URL parent depth does not match its path");
		Arc::make_mut(&mut auth).parent = parent;
		Ok((path, auth))
	}

	fn normalize_ports(
		kind: AuthKind,
		uri: Option<usize>,
		urn: Option<usize>,
		path: &PathCow,
	) -> Result<(usize, usize)> {
		Ok(match kind {
			AuthKind::Regular => {
				ensure!(uri.is_none() && urn.is_none(), "Regular scheme cannot have ports");
				(path.name().is_some() as usize, path.name().is_some() as usize)
			}
			AuthKind::Search => {
				let (uri, urn) = (uri.unwrap_or(0), urn.unwrap_or(0));
				ensure!(uri == urn, "Search scheme requires URI and URN to be equal");
				(uri, urn)
			}
			AuthKind::Mount => (uri.unwrap_or(0), urn.unwrap_or(0)),
			AuthKind::Hub | AuthKind::Scope | AuthKind::Sftp => {
				let uri = uri.unwrap_or(path.name().is_some() as usize);
				let urn = urn.unwrap_or(path.name().is_some() as usize);
				(uri, urn)
			}
		})
	}

	pub fn retrieve_ports(url: Url) -> (usize, usize) {
		match url {
			Url::Regular(loc) => (loc.file_name().is_some() as usize, loc.file_name().is_some() as usize),
			Url::Search { loc, .. } | Url::Mount { loc, .. } | Url::Hub { loc, .. } => {
				(loc.uri().components().count(), loc.urn().components().count())
			}
			Url::Scope { loc, .. } | Url::Sftp { loc, .. } => {
				(loc.uri().components().count(), loc.urn().components().count())
			}
		}
	}
}
