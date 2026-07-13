use std::{fmt, sync::Arc};

use anyhow::{Result, anyhow};
use percent_encoding::percent_decode_str;
use serde::Deserialize;
use yazi_shim::{SStr, cell::RoCell};

use crate::auth::{AuthInventory, AuthKind, Encode, Scheme};

pub(super) static DEFAULT_ARC: RoCell<Arc<Auth>> = RoCell::new();

#[derive(Debug, Deserialize, Eq, Hash, PartialEq)]
pub struct Auth {
	pub kind:   AuthKind,
	pub scheme: Scheme,
	pub domain: SStr,
}

impl Default for Auth {
	fn default() -> Self { Self::DEFAULT }
}

impl fmt::Display for Auth {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { Encode(self, false).fmt(f) }
}

impl Auth {
	pub const DEFAULT: Self =
		Self { kind: AuthKind::Regular, scheme: Scheme::Regular, domain: SStr::Borrowed("") };

	pub fn default_arc() -> Arc<Self> { DEFAULT_ARC.clone() }

	pub fn new(kind: AuthKind, scheme: Scheme, domain: impl Into<SStr>) -> Arc<Self> {
		Arc::new(Self { kind, scheme, domain: domain.into() })
	}

	pub fn get(scheme: &Scheme, domain: &str) -> Option<Arc<Self>> {
		match scheme {
			Scheme::Regular => Some(Self::default_arc()),
			Scheme::Search => Some(Self::search(domain)),
			_ => inventory::iter::<AuthInventory>().find_map(|entry| (entry.get)(scheme, domain)),
		}
	}

	pub fn search(query: impl Into<String>) -> Arc<Self> {
		Self::new(AuthKind::Search, Scheme::Search, query.into())
	}

	pub fn covariant(&self, other: &Self) -> bool {
		!self.kind.is_virtual() && !other.kind.is_virtual() || self == other
	}

	pub fn parse_cache(cache: &str) -> Result<Arc<Self>> {
		let (kind, rest) = cache.split_once('_').ok_or_else(|| anyhow!("invalid cache: {cache}"))?;
		let (scheme, domain) = rest.split_once('_').ok_or_else(|| anyhow!("invalid cache: {cache}"))?;

		Ok(Arc::new(Self {
			kind:   kind.parse()?,
			scheme: scheme.parse()?,
			domain: percent_decode_str(domain).decode_utf8()?.into_owned().into(),
		}))
	}
}
