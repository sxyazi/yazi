use std::{fmt, sync::Arc};

use serde::Deserialize;
use yazi_shim::cell::RoCell;

use crate::{auth::{AuthInventory, AuthKind, Domain, EncodeAuth, Scheme}, path::{Component, Components}};

pub(super) static DEFAULT_ARC: RoCell<Arc<Auth>> = RoCell::new();

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq)]
pub struct Auth {
	pub kind:   AuthKind,
	pub scheme: Scheme,
	pub domain: Domain<'static>,
	#[serde(default)]
	pub parent: Option<Arc<Auth>>,
}

impl Default for Auth {
	fn default() -> Self { Self::DEFAULT }
}

impl fmt::Display for Auth {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { EncodeAuth(self, false).fmt(f) }
}

impl Auth {
	pub(crate) const DEFAULT: Self = Self {
		kind:   AuthKind::Regular,
		scheme: Scheme::Regular,
		domain: Domain::EMPTY,
		parent: None,
	};

	pub(crate) fn default_arc() -> Arc<Self> { DEFAULT_ARC.clone() }

	pub fn new<'a>(kind: AuthKind, scheme: Scheme, domain: impl Into<Domain<'a>>) -> Arc<Self> {
		Arc::new(Self { kind, scheme, domain: domain.into().into_owned(), parent: None })
	}

	pub(crate) fn search<'a>(query: impl Into<Domain<'a>>) -> Arc<Self> {
		Self::new(AuthKind::Search, Scheme::Search, query)
	}

	pub(crate) fn get(scheme: &Scheme, domain: &Domain<'_>) -> Option<Arc<Self>> {
		match scheme {
			Scheme::Regular => Some(Self::default_arc()),
			Scheme::Search => Some(Self::search(domain)),
			_ => inventory::iter::<AuthInventory>().find_map(|entry| (entry.get)(scheme, domain)),
		}
	}

	fn child(self: Arc<Self>) -> Arc<Self> {
		Arc::new(Self {
			kind:   self.kind,
			scheme: self.scheme.clone(),
			domain: Domain::default(),
			parent: Some(self),
		})
	}

	pub(crate) fn descend<'a, C>(mut self: Arc<Self>, components: C) -> Arc<Self>
	where
		C: Into<Components<'a>>,
	{
		for component in components.into() {
			match component {
				Component::RootDir => self = Self::new(self.kind, self.scheme.clone(), Domain::EMPTY),
				c if c.has_auth() => self = self.child(),
				_ => {}
			}
		}
		self
	}

	pub(crate) fn parent_at(mut self: &Arc<Self>, depth: usize) -> &Arc<Self> {
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

	pub(crate) fn parent_depth(&self) -> usize {
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
}
