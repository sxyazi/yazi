use std::{hash::{Hash, Hasher}, ops::Deref, sync::Arc};

use serde::Deserialize;

use super::{Auth, AuthError, AuthInventory, AuthKind, Scheme, View};
use crate::{domain::Domain, path::{Component, Components}};

#[repr(transparent)]
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(from = "Auth")]
pub struct AuthArc(Option<Arc<Auth>>);

impl Deref for AuthArc {
	type Target = Auth;

	fn deref(&self) -> &Self::Target { self.0.as_deref().unwrap_or(&Auth::DEFAULT) }
}

impl Eq for AuthArc {}

impl PartialEq for AuthArc {
	fn eq(&self, other: &Self) -> bool { self.deref() == other.deref() }
}

impl Hash for AuthArc {
	fn hash<H: Hasher>(&self, state: &mut H) { self.deref().hash(state) }
}

impl From<Auth> for AuthArc {
	fn from(auth: Auth) -> Self { Self(Some(Arc::new(auth))) }
}

impl AuthArc {
	pub(crate) const DEFAULT: Self = Self(None);

	#[inline]
	pub fn new<'a>(kind: AuthKind, scheme: Scheme, domain: impl Into<Domain<'a>>) -> Self {
		Auth::new(kind, scheme, domain).into()
	}

	pub(crate) fn get<'a>(scheme: &'a Scheme, domain: &'a Domain<'a>) -> Result<Self, AuthError<'a>> {
		match scheme {
			Scheme::Regular => Some(Default::default()),
			_ => inventory::iter::<AuthInventory>().find_map(|entry| (entry.get)(scheme, domain)),
		}
		.ok_or(AuthError { scheme, domain })
	}

	#[inline]
	pub fn is_regular(&self) -> bool { self.0.is_none() || self.kind.is_regular() }

	#[inline]
	pub fn make_mut(&mut self) -> &mut Auth {
		Arc::make_mut(self.0.get_or_insert_with(|| Arc::new(Auth::DEFAULT)))
	}

	fn child(self) -> Self {
		Auth {
			kind:   self.kind,
			scheme: self.scheme.clone(),
			domain: Domain::default(),
			parent: Some(self),
			view:   Default::default(),
		}
		.into()
	}

	pub(crate) fn descend<'a, C>(mut self, components: C) -> Self
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

	pub(crate) fn parent_at(mut self: &Self, depth: usize) -> &Self {
		for _ in 0..depth {
			self = self.parent.as_ref().expect("Auth parent depth out of bounds");
		}
		self
	}

	pub(crate) fn with_view(mut self, view: View) -> Self {
		debug_assert!(self.kind.is_view());
		self.make_mut().view = view.into();
		self
	}

	pub fn with_parent_depth(mut self, depth: usize) -> Self {
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
			parent = Some(Self::from(Auth {
				kind: self.kind,
				scheme: self.scheme.clone(),
				domain: Domain::default(),
				parent,
				view: Default::default(),
			}));
		}

		self.make_mut().parent = parent;
		self
	}
}
