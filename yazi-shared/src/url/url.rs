use std::{borrow::Cow, ffi::OsStr, path::Path};

use anyhow::Result;
use hashbrown::Equivalent;
use serde::Serialize;

use crate::{auth::{AuthArc, AuthKind}, loc::{Loc, LocBuf}, path::{DynPath, DynPathRef, EndsWithError, JoinError, PathBufDyn, PathDyn, PathDynError, PathLike, StartsWithError, StripPrefixError}, spec::{Encode as EncodeSpec, ParsedSpec, Spec}, strand::{AsStrand, Strand}, url::{AsUrl, Components, UrlBuf, UrlCow}};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Url<'a> {
	Os { loc: Loc<'a>, auth: &'a AuthArc },
	Unix { loc: Loc<'a, &'a typed_path::UnixPath>, auth: &'a AuthArc },
}

impl PartialEq<UrlBuf> for Url<'_> {
	fn eq(&self, other: &UrlBuf) -> bool { *self == other.as_url() }
}

impl Equivalent<UrlBuf> for Url<'_> {
	fn equivalent(&self, key: &UrlBuf) -> bool { self == key }
}

impl Serialize for Url<'_> {
	fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
		match (self.is_regular(), self.loc().to_str()) {
			(true, Ok(s)) => serializer.serialize_str(s),
			(false, Ok(s)) => serializer.serialize_str(&format!("{}{s}", EncodeSpec(*self))),
			(_, Err(_)) => serializer.collect_str(&self.encode()),
		}
	}
}

impl<'a> Url<'a> {
	#[inline]
	pub fn as_local(self) -> Option<&'a Path> {
		self.loc().as_os().ok().filter(|_| self.auth().is_local())
	}

	#[inline]
	pub fn auth(self) -> &'a AuthArc {
		match self {
			Self::Os { auth, .. } | Self::Unix { auth, .. } => auth,
		}
	}

	fn auth_at(self, base: Self) -> Option<&'a AuthArc> {
		if !self.kind().is_hub() || !base.kind().is_hub() {
			return self.auth().covariant(base.auth()).then_some(self.auth());
		}

		let depth =
			self.loc().components().auth_depth().checked_sub(base.loc().components().auth_depth())?;
		Some(self.auth().parent_at(depth))
	}

	#[inline]
	pub fn as_regular(self) -> Result<Self, PathDynError> {
		Ok(Self::Os { loc: Loc::bare(self.loc().as_os()?), auth: &AuthArc::DEFAULT })
	}

	pub(crate) fn base(self) -> Self {
		match self {
			Self::Os { loc, auth } if auth.kind.is_hub() => Self::Os {
				loc:  Loc::bare(loc.base()),
				auth: auth.parent_at(loc.uri().dyn_path().components().auth_depth()),
			},
			Self::Os { loc, auth } => Self::Os { loc: Loc::saturated(loc.base(), auth.kind), auth },
			Self::Unix { loc, auth } => Self::Unix { loc: Loc::saturated(loc.base(), auth.kind), auth },
		}
	}

	#[inline]
	pub fn components(self) -> Components<'a> { Components::from(self) }

	#[inline]
	pub(crate) fn covariant(self, other: impl AsUrl) -> bool {
		let other = other.as_url();
		self.loc() == other.loc() && self.auth().covariant(other.auth())
	}

	#[inline]
	pub(crate) fn ext(self) -> Option<Strand<'a>> { self.loc().ext() }

	#[inline]
	pub(crate) fn has_base(self) -> bool {
		match self {
			Self::Os { loc, .. } => loc.has_base(),
			Self::Unix { loc, .. } => loc.has_base(),
		}
	}

	#[inline]
	pub(crate) fn has_root(self) -> bool { self.loc().has_root() }

	#[inline]
	pub(crate) fn has_trail(self) -> bool {
		match self {
			Self::Os { loc, .. } => loc.has_trail(),
			Self::Unix { loc, .. } => loc.has_trail(),
		}
	}

	#[inline]
	pub(crate) fn is_absolute(self) -> bool { self.loc().is_absolute() }

	#[inline]
	pub fn is_regular(self) -> bool { self.auth().is_regular() }

	#[inline]
	pub fn is_view(self) -> bool { self.kind().is_view() }

	#[inline]
	pub(crate) fn key(self) -> PathDyn<'a> {
		if self.kind().is_hub() {
			PathDyn::Unix(typed_path::UnixPath::new(&self.auth().domain))
		} else {
			self.urn()
		}
	}

	#[inline]
	pub fn kind(self) -> AuthKind { self.auth().kind }

	#[inline]
	pub fn loc(self) -> PathDyn<'a> {
		match self {
			Self::Os { loc, .. } => loc.dyn_path(),
			Self::Unix { loc, .. } => loc.dyn_path(),
		}
	}

	#[inline]
	pub fn name(self) -> Option<Strand<'a>> { self.loc().name() }

	#[inline]
	pub fn os_str(self) -> Cow<'a, OsStr> { self.components().os_str() }

	#[inline]
	pub fn pair(self) -> Option<(Self, PathDyn<'a>)> {
		let key = self.key();
		(!key.is_empty()).then_some((self.trail(), key))
	}

	pub fn parent(self) -> Option<Self> {
		let auth = self.auth();
		let uri = self.uri();
		let kind = auth.kind;

		Some(match self {
			// View portal
			Self::Os { .. } | Self::Unix { .. } if kind.is_view() && uri.is_empty() => {
				self.base().physical().parent()?
			}

			// View item
			Self::Os { loc, .. } if kind.is_view() => Self::Os { loc: loc.ascend()?, auth },
			Self::Unix { loc, .. } if kind.is_view() => Self::Unix { loc: loc.ascend()?, auth },

			// Mount portal
			Self::Os { loc, .. } if kind.is_mount() && uri.is_empty() => Self::regular(loc.parent()?),
			Self::Os { loc, .. } if kind.is_mount() && uri.components().nth(1).is_none() => {
				Self::Os { loc: Loc::zeroed(loc.parent()?), auth }
			}

			// Mount item
			Self::Os { loc, .. } if kind.is_mount() => {
				Self::Os { loc: Loc::floated(loc.parent()?, loc.base()), auth }
			}

			// Hub
			Self::Os { loc, .. } if kind.is_hub() => {
				Self::Os { loc: Loc::bare(loc.parent()?), auth: auth.parent.as_ref()? }
			}

			// Regular
			Self::Os { loc, .. } => Self::Os { loc: Loc::bare(loc.parent()?), auth },

			// Scope / Sftp
			Self::Unix { loc, .. } => Self::Unix { loc: Loc::bare(loc.parent()?), auth },
		})
	}

	pub fn physical(self) -> Self {
		let Some(auth) = self.auth().view.auth() else { return self };

		match self {
			Self::Os { loc, .. } => Self::Os { loc: Loc::bare(loc.as_inner()), auth },
			Self::Unix { loc, .. } => Self::Unix { loc: Loc::bare(loc.as_inner()), auth },
		}
	}

	#[inline]
	pub fn regular<T: AsRef<Path> + ?Sized>(path: &'a T) -> Self {
		Self::Os { loc: Loc::bare(path.as_ref()), auth: &AuthArc::DEFAULT }
	}

	pub fn spec(self) -> Spec {
		let auth = self.auth().clone();
		let (uri, urn) = Spec::retrieve_ports(self);

		Spec { auth, uri, urn }
	}

	#[inline]
	pub(crate) fn stem(self) -> Option<Strand<'a>> { self.loc().stem() }

	#[inline]
	pub fn to_owned(self) -> UrlBuf { self.into() }

	pub(crate) fn trail(self) -> Self {
		let auth = self.auth();
		let uri = self.uri();
		let kind = auth.kind;

		match self {
			// View portal
			Self::Os { loc, .. } if kind.is_view() && uri.is_empty() => {
				Self::Os { loc: Loc::zeroed(loc.trail()), auth }
			}
			Self::Unix { loc, .. } if kind.is_view() && uri.is_empty() => {
				Self::Unix { loc: Loc::zeroed(loc.trail()), auth }
			}

			// View item
			Self::Os { loc, .. } if kind.is_view() => {
				Self::Os { loc: Loc::new(loc.trail(), loc.base(), loc.base()), auth }
			}
			Self::Unix { loc, .. } if kind.is_view() => {
				Self::Unix { loc: Loc::new(loc.trail(), loc.base(), loc.base()), auth }
			}

			// Mount portal
			Self::Os { loc, .. } if kind.is_mount() && uri.is_empty() => {
				Self::Os { loc: Loc::zeroed(loc.trail()), auth }
			}

			// Mount item
			Self::Os { loc, .. } if kind.is_mount() => {
				Self::Os { loc: Loc::new(loc.trail(), loc.base(), loc.base()), auth }
			}

			// Hub
			Self::Os { loc, .. } if kind.is_hub() => Self::Os {
				loc:  Loc::bare(loc.trail()),
				auth: auth.parent_at(loc.urn().dyn_path().components().auth_depth()),
			},

			// Regular
			Self::Os { loc, .. } => Self::Os { loc: Loc::bare(loc.trail()), auth },

			// Scope / Sftp
			Self::Unix { loc, .. } => Self::Unix { loc: Loc::bare(loc.trail()), auth },
		}
	}

	pub(crate) fn triple(self) -> (PathDyn<'a>, PathDyn<'a>, PathDyn<'a>) {
		match self {
			Self::Os { loc, .. } => {
				let (base, rest, urn) = loc.triple();
				(base.dyn_path(), rest.dyn_path(), urn.dyn_path())
			}
			Self::Unix { loc, .. } => {
				let (base, rest, urn) = loc.triple();
				(base.dyn_path(), rest.dyn_path(), urn.dyn_path())
			}
		}
	}

	#[inline]
	pub(crate) fn try_ends_with(self, child: impl AsUrl) -> Result<bool, EndsWithError> {
		let child = child.as_url();
		Ok(self.loc().try_ends_with(child.loc())? && self.auth().covariant(child.auth()))
	}

	pub fn try_join(self, path: impl AsStrand) -> Result<UrlBuf, JoinError> {
		let path = path.as_strand();
		let joined = self.loc().try_join(path)?;

		let auth = self.auth().clone();
		let kind = auth.kind;

		Ok(match self {
			// View item
			Self::Os { loc, .. } if kind.is_view() && joined.try_starts_with(loc.trail())? => {
				UrlBuf::Os { loc: LocBuf::new(joined.try_into()?, loc.base(), loc.trail()), auth }
			}
			Self::Unix { loc, .. } if kind.is_view() && joined.try_starts_with(loc.trail())? => {
				UrlBuf::Unix { loc: LocBuf::new(joined.into_unix()?, loc.base(), loc.trail()), auth }
			}

			// View portal
			Self::Unix { .. } if kind.is_view() => {
				UrlBuf::Unix { loc: LocBuf::zeroed(joined.into_unix()?), auth }
			}

			// Mount item
			Self::Os { loc, .. } if kind.is_mount() && joined.try_starts_with(loc.base())? => {
				UrlBuf::Os { loc: LocBuf::floated(joined.try_into()?, loc.base()), auth }
			}

			// View portal / Mount portal
			Self::Os { .. } if kind.is_view() || kind.is_mount() => {
				UrlBuf::Os { loc: LocBuf::zeroed(joined.into_os()?), auth }
			}

			// Hub
			Self::Os { .. } if kind.is_hub() => {
				UrlBuf::Os { loc: joined.into_os()?.into(), auth: auth.descend(path.as_os_path()?) }
			}

			// Regular
			Self::Os { .. } => UrlBuf::Os { loc: joined.into_os()?.into(), auth },

			// Scope / Sftp
			Self::Unix { .. } => UrlBuf::Unix { loc: joined.into_unix()?.into(), auth },
		})
	}

	pub(crate) fn try_replace<'b>(self, take: usize, to: impl DynPathRef<'b>) -> Result<UrlCow<'b>> {
		self.try_replace_impl(take, to.dyn_path_ref())
	}

	fn try_replace_impl<'b>(self, take: usize, rep: PathDyn<'b>) -> Result<UrlCow<'b>> {
		let b = rep.encoded_bytes();
		if take == 0 || ParsedSpec::parse(b)?.has_scheme() {
			return UrlCow::try_from(b);
		}

		let loc = self.loc();
		let mut path = PathBufDyn::from_components(loc.kind(), loc.components().take(take - 1))?;
		path.try_push(rep)?;

		let auth = self.auth().clone();
		let kind = auth.kind;
		Ok(UrlCow::from(match self {
			// View item / Mount item
			Self::Os { loc, .. }
				if (kind.is_view() || kind.is_mount()) && path.try_starts_with(loc.trail())? =>
			{
				UrlBuf::Os { loc: LocBuf::new(path.into_os()?, loc.base(), loc.trail()), auth }
			}

			// Hub
			Self::Os { loc, .. } if kind.is_hub() && path.try_starts_with(loc.trail())? => UrlBuf::Os {
				auth: auth.with_parent_depth(path.components().auth_depth()),
				loc:  LocBuf::new(path.into_os()?, loc.base(), loc.trail()),
			},

			// View item / Scope / Sftp
			Self::Unix { loc, .. } if path.try_starts_with(loc.trail())? => {
				UrlBuf::Unix { loc: LocBuf::new(path.into_unix()?, loc.base(), loc.trail()), auth }
			}

			// Hub
			Self::Os { .. } if kind.is_hub() => UrlBuf::Os {
				auth: auth.with_parent_depth(path.components().auth_depth()),
				loc:  LocBuf::saturated(path.into_os()?, kind),
			},

			// Regular / View portal / Mount portal
			Self::Os { .. } => UrlBuf::Os { loc: LocBuf::saturated(path.into_os()?, kind), auth },

			// View portal / Scope / Sftp
			Self::Unix { .. } => UrlBuf::Unix { loc: LocBuf::saturated(path.into_unix()?, kind), auth },
		}))
	}

	pub(crate) fn try_starts_with(self, base: impl AsUrl) -> Result<bool, StartsWithError> {
		let base = base.as_url();
		Ok(
			self.loc().try_starts_with(base.loc())?
				&& self.auth_at(base).is_some_and(|a| a == base.auth()),
		)
	}

	pub(crate) fn try_strip_prefix(self, base: impl AsUrl) -> Result<PathDyn<'a>, StripPrefixError> {
		use StripPrefixError::{Exotic, NotPrefix};

		let base = base.as_url();
		let prefix = self.loc().try_strip_prefix(base.loc())?;
		if self.auth_at(base).is_some_and(|a| a == base.auth()) {
			return Ok(prefix);
		}

		match (self.kind(), base.kind()) {
			// A mount portal is the local source file until it gains an inner URI.
			(AuthKind::Regular | AuthKind::View, AuthKind::Mount) => {
				base.uri().is_empty().then_some(prefix).ok_or(NotPrefix)
			}
			(AuthKind::Mount, AuthKind::Regular | AuthKind::View) => {
				self.uri().is_empty().then_some(prefix).ok_or(NotPrefix)
			}
			_ => Err(Exotic),
		}
	}

	#[inline]
	pub(crate) fn uri(self) -> PathDyn<'a> {
		match self {
			Self::Os { loc, .. } => loc.uri().dyn_path(),
			Self::Unix { loc, .. } => loc.uri().dyn_path(),
		}
	}

	#[inline]
	pub fn urn(self) -> PathDyn<'a> {
		match self {
			Self::Os { loc, .. } => loc.urn().dyn_path(),
			Self::Unix { loc, .. } => loc.urn().dyn_path(),
		}
	}
}
