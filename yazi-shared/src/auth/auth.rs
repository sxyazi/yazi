use std::{fmt, hash::{Hash, Hasher}, sync::Arc};

use anyhow::{Result, anyhow};
use percent_encoding::percent_decode_str;
use serde::Deserialize;
use yazi_shim::cell::RoCell;

use crate::{auth::{AuthInventory, AuthKind, Domain, EncodeAuth, Scheme}, path::{Component, Components}};

pub(super) static DEFAULT_ARC: RoCell<Arc<Auth>> = RoCell::new();

#[derive(Clone, Debug, Deserialize)]
pub struct Auth {
	pub kind:   AuthKind,
	pub scheme: Scheme,
	pub domain: Domain<'static>,
	#[serde(default)]
	pub parent: Option<Arc<Auth>>,
}

impl PartialEq for Auth {
	fn eq(&self, other: &Self) -> bool {
		self.kind == other.kind && self.scheme == other.scheme && self.domain == other.domain
	}
}

impl Eq for Auth {}

impl Hash for Auth {
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.kind.hash(state);
		self.scheme.hash(state);
		self.domain.hash(state);
	}
}

impl Default for Auth {
	fn default() -> Self { Self::DEFAULT }
}

impl fmt::Display for Auth {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { EncodeAuth(self, false).fmt(f) }
}

impl Auth {
	pub const DEFAULT: Self = Self {
		kind:   AuthKind::Regular,
		scheme: Scheme::Regular,
		domain: Domain::EMPTY,
		parent: None,
	};

	pub fn default_arc() -> Arc<Self> { DEFAULT_ARC.clone() }

	pub fn new<'a>(kind: AuthKind, scheme: Scheme, domain: impl Into<Domain<'a>>) -> Arc<Self> {
		Arc::new(Self { kind, scheme, domain: domain.into().into_owned(), parent: None })
	}

	pub fn search<'a>(query: impl Into<Domain<'a>>) -> Arc<Self> {
		Self::new(AuthKind::Search, Scheme::Search, query)
	}

	pub fn get(scheme: &Scheme, domain: &Domain<'_>) -> Option<Arc<Self>> {
		match scheme {
			Scheme::Regular => Some(Self::default_arc()),
			Scheme::Search => Some(Self::search(domain)),
			_ => inventory::iter::<AuthInventory>().find_map(|entry| (entry.get)(scheme, domain)),
		}
	}

	pub fn child(self: Arc<Self>) -> Arc<Self> {
		Arc::new(Self {
			kind:   self.kind,
			scheme: self.scheme.clone(),
			domain: Domain::default(),
			parent: Some(self),
		})
	}

	pub fn root(mut self: &Arc<Self>) -> &Arc<Self> {
		while let Some(parent) = &self.parent {
			self = parent;
		}
		self
	}

	pub fn descend<'a>(mut self: Arc<Self>, components: impl Into<Components<'a>>) -> Arc<Self> {
		for component in components.into() {
			match component {
				Component::RootDir => self = Self::new(self.kind, self.scheme.clone(), Domain::EMPTY),
				c if c.has_auth() => self = self.child(),
				_ => {}
			}
		}
		self
	}

	pub fn parent_at(mut self: &Arc<Self>, depth: usize) -> &Arc<Self> {
		for _ in 0..depth {
			self = self.parent.as_ref().expect("Auth parent depth out of bounds");
		}
		self
	}

	pub fn with_parent_depth(mut self: Arc<Self>, depth: usize) -> Arc<Self> {
		let current = self.parent_depth();
		if current == depth {
			return self;
		}

		let mut parent = if current < depth {
			self.parent.clone()
		} else {
			self.parent_at(current - depth).parent.clone()
		};

		for _ in current..depth {
			parent = Some(Arc::new(Self {
				kind: self.kind,
				scheme: self.scheme.clone(),
				domain: Domain::default(),
				parent,
			}));
		}

		Arc::make_mut(&mut self).parent = parent;
		self
	}

	pub fn parent_depth(&self) -> usize {
		let mut depth = 0;
		let mut parent = self.parent.as_deref();
		while let Some(auth) = parent {
			depth += 1;
			parent = auth.parent.as_deref();
		}
		depth
	}

	pub fn covariant(&self, other: &Self) -> bool {
		!self.kind.is_virtual() && !other.kind.is_virtual() || self == other
	}

	pub fn same_service(&self, other: &Self) -> bool {
		self.covariant(other)
			|| self.kind == AuthKind::Hub && other.kind == AuthKind::Hub && self.scheme == other.scheme
	}

	pub fn parse_cache(cache: &str) -> Result<Arc<Self>> {
		let (kind, rest) = cache.split_once('_').ok_or_else(|| anyhow!("invalid cache: {cache}"))?;
		let (scheme, domain) = rest.split_once('_').ok_or_else(|| anyhow!("invalid cache: {cache}"))?;

		Ok(Self::new(kind.parse()?, scheme.parse()?, percent_decode_str(domain).decode_utf8()?))
	}
}
