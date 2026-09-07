use anyhow::{Result, bail, ensure};
use serde::{Deserialize, Deserializer, de::Error};

use crate::{auth::{AuthArc, AuthKind, Scheme, ViewBox}, domain::Domain, path::PathKind};

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Auth {
	pub kind:   AuthKind,
	pub scheme: Scheme,
	pub domain: Domain<'static>,
	pub parent: Option<AuthArc>,
	pub view:   ViewBox,
}

impl Default for Auth {
	fn default() -> Self { Self::DEFAULT }
}

impl Auth {
	pub(crate) const DEFAULT: Self = Self {
		kind:   AuthKind::Regular,
		scheme: Scheme::Regular,
		domain: Domain::EMPTY,
		parent: None,
		view:   ViewBox::DEFAULT,
	};

	pub fn new<'a>(kind: AuthKind, scheme: Scheme, domain: impl Into<Domain<'a>>) -> Self {
		Self {
			kind,
			scheme,
			domain: domain.into().into_owned(),
			parent: None,
			view: Default::default(),
		}
	}

	#[inline]
	pub fn is_local(&self) -> bool { self.kind.is_regular() || self.view.is_local() }

	#[inline]
	pub fn is_remote(&self) -> bool { self.kind.is_sftp() || self.view.is_remote() }

	#[inline]
	pub fn physical(&self) -> &Self { self.view.auth().map_or(self, |a| a) }

	#[inline]
	pub(crate) fn path_kind(&self) -> Result<PathKind> { self.physical().kind.try_into() }

	#[inline]
	pub fn covariant(&self, other: &Self) -> bool { self.physical() == other.physical() }

	pub fn same_service(&self, other: &Self) -> bool {
		self.covariant(other)
			|| self.kind.is_hub() && other.kind.is_hub() && self.scheme == other.scheme
	}

	pub(crate) fn validate(&self) -> Result<()> {
		match (self.kind.is_view(), self.view.auth()) {
			(true, None) => bail!("View auth requires view metadata"),
			(false, Some(_)) => bail!("Non-view auth cannot have view metadata"),
			(true, Some(source)) => {
				ensure!(source.is_regular() || source.kind.is_sftp(), "View source must be Regular or Sftp")
			}
			_ => {}
		}
		Ok(())
	}

	pub(crate) fn parent_depth(&self) -> usize {
		let mut depth = 0;
		let mut parent = self.parent.as_ref();
		while let Some(auth) = parent {
			depth += 1;
			parent = auth.parent.as_ref();
		}
		depth
	}
}

impl<'de> Deserialize<'de> for Auth {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		#[derive(Deserialize)]
		struct Shadow {
			kind:   AuthKind,
			scheme: Scheme,
			domain: Domain<'static>,
			#[serde(default)]
			parent: Option<AuthArc>,
			#[serde(default)]
			view:   ViewBox,
		}

		let Shadow { kind, scheme, domain, parent, view } = Shadow::deserialize(deserializer)?;
		let auth = Self { kind, scheme, domain, parent, view };

		auth.validate().map_err(D::Error::custom)?;
		Ok(auth)
	}
}
