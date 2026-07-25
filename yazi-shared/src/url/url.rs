use std::{borrow::Cow, ffi::OsStr, fmt::{Debug, Formatter}, path::{Path, PathBuf}, sync::Arc};

use anyhow::Result;
use hashbrown::Equivalent;
use serde::Serialize;

use super::Encode as EncodeUrl;
use crate::{auth::{Auth, AuthKind}, loc::{Loc, LocBuf}, path::{DynPath, DynPathRef, EndsWithError, JoinError, PathBufDyn, PathDyn, PathDynError, PathLike, StartsWithError, StripPrefixError, StripSuffixError}, spec::{EncodeSpec, ParsedSpec, Spec}, strand::{AsStrand, Strand}, url::{AsUrl, Components, UrlBuf, UrlCow}};

#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub enum Url<'a> {
	Regular(Loc<'a>),
	Search { loc: Loc<'a>, auth: &'a Arc<Auth> },
	Mount { loc: Loc<'a>, auth: &'a Arc<Auth> },
	Hub { loc: Loc<'a>, auth: &'a Arc<Auth> },
	Scope { loc: Loc<'a, &'a typed_path::UnixPath>, auth: &'a Arc<Auth> },
	Sftp { loc: Loc<'a, &'a typed_path::UnixPath>, auth: &'a Arc<Auth> },
}

// --- Eq
impl PartialEq<UrlBuf> for Url<'_> {
	fn eq(&self, other: &UrlBuf) -> bool { *self == other.as_url() }
}

// --- Hash
impl Equivalent<UrlBuf> for Url<'_> {
	fn equivalent(&self, key: &UrlBuf) -> bool { self == key }
}

// --- Debug
impl Debug for Url<'_> {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		if self.is_regular() {
			write!(f, "{}", self.loc().display())
		} else {
			write!(f, "{}{}", EncodeSpec(*self), self.loc().display())
		}
	}
}

impl Serialize for Url<'_> {
	fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
		let (kind, loc) = (self.kind(), self.loc());
		match (kind == AuthKind::Regular, loc.to_str()) {
			(true, Ok(s)) => serializer.serialize_str(s),
			(false, Ok(s)) => serializer.serialize_str(&format!("{}{s}", EncodeSpec(*self))),
			(_, Err(_)) => serializer.collect_str(&EncodeUrl(*self)),
		}
	}
}

impl<'a> Url<'a> {
	#[inline]
	pub fn as_local(self) -> Option<&'a Path> {
		self.loc().as_os().ok().filter(|_| self.kind().is_local())
	}

	#[inline]
	pub fn auth(self) -> &'a Auth {
		match self {
			Self::Regular(_) => &Auth::DEFAULT,
			Self::Search { auth, .. }
			| Self::Mount { auth, .. }
			| Self::Hub { auth, .. }
			| Self::Scope { auth, .. }
			| Self::Sftp { auth, .. } => auth,
		}
	}

	fn auth_at(self, base: Self) -> Option<&'a Auth> {
		let (Self::Hub { auth, .. }, Self::Hub { .. }) = (self, base) else {
			return self.auth().covariant(base.auth()).then_some(self.auth());
		};

		let depth =
			self.loc().components().auth_depth().checked_sub(base.loc().components().auth_depth())?;
		Some(auth.parent_at(depth))
	}

	#[inline]
	pub fn as_regular(self) -> Result<Self, PathDynError> {
		Ok(Self::Regular(Loc::bare(self.loc().as_os()?)))
	}

	pub fn base(self) -> Self {
		match self {
			Self::Regular(loc) => Self::Regular(Loc::bare(loc.base())),
			Self::Search { loc, auth } => Self::Search { loc: Loc::zeroed(loc.base()), auth },
			Self::Mount { loc, auth } => Self::Mount { loc: Loc::zeroed(loc.base()), auth },
			Self::Hub { loc, auth } => Self::Hub {
				loc:  Loc::bare(loc.base()),
				auth: auth.parent_at(loc.uri().dyn_path().components().auth_depth()),
			},
			Self::Scope { loc, auth } => Self::Scope { loc: Loc::bare(loc.base()), auth },
			Self::Sftp { loc, auth } => Self::Sftp { loc: Loc::bare(loc.base()), auth },
		}
	}

	#[inline]
	pub fn components(self) -> Components<'a> { Components::from(self) }

	#[inline]
	pub fn covariant(self, other: impl AsUrl) -> bool {
		let other = other.as_url();
		self.loc() == other.loc() && self.auth().covariant(other.auth())
	}

	#[inline]
	pub fn ext(self) -> Option<Strand<'a>> {
		Some(match self {
			Self::Regular(loc) => loc.extension()?.as_strand(),
			Self::Search { loc, .. } => loc.extension()?.as_strand(),
			Self::Mount { loc, .. } => loc.extension()?.as_strand(),
			Self::Hub { loc, .. } => loc.extension()?.as_strand(),
			Self::Scope { loc, .. } => loc.extension()?.as_strand(),
			Self::Sftp { loc, .. } => loc.extension()?.as_strand(),
		})
	}

	#[inline]
	pub fn has_base(self) -> bool {
		match self {
			Self::Regular(loc) => loc.has_base(),
			Self::Search { loc, .. } => loc.has_base(),
			Self::Mount { loc, .. } => loc.has_base(),
			Self::Hub { loc, .. } => loc.has_base(),
			Self::Scope { loc, .. } => loc.has_base(),
			Self::Sftp { loc, .. } => loc.has_base(),
		}
	}

	#[inline]
	pub fn has_root(self) -> bool { self.loc().has_root() }

	#[inline]
	pub fn has_trail(self) -> bool {
		match self {
			Self::Regular(loc) => loc.has_trail(),
			Self::Search { loc, .. } => loc.has_trail(),
			Self::Mount { loc, .. } => loc.has_trail(),
			Self::Hub { loc, .. } => loc.has_trail(),
			Self::Scope { loc, .. } => loc.has_trail(),
			Self::Sftp { loc, .. } => loc.has_trail(),
		}
	}

	#[inline]
	pub fn is_absolute(self) -> bool { self.loc().is_absolute() }

	#[inline]
	pub fn is_regular(self) -> bool { matches!(self, Self::Regular(_)) }

	#[inline]
	pub fn is_search(self) -> bool { matches!(self, Self::Search { .. }) }

	#[inline]
	pub fn kind(self) -> AuthKind { self.auth().kind }

	#[inline]
	pub fn loc(self) -> PathDyn<'a> {
		match self {
			Self::Regular(loc) => loc.dyn_path(),
			Self::Search { loc, .. } => loc.dyn_path(),
			Self::Mount { loc, .. } => loc.dyn_path(),
			Self::Hub { loc, .. } => loc.dyn_path(),
			Self::Scope { loc, .. } => loc.dyn_path(),
			Self::Sftp { loc, .. } => loc.dyn_path(),
		}
	}

	#[inline]
	pub fn name(self) -> Option<Strand<'a>> {
		Some(match self {
			Self::Regular(loc) => loc.file_name()?.as_strand(),
			Self::Search { loc, .. } => loc.file_name()?.as_strand(),
			Self::Mount { loc, .. } => loc.file_name()?.as_strand(),
			Self::Hub { loc, .. } => loc.file_name()?.as_strand(),
			Self::Scope { loc, .. } => loc.file_name()?.as_strand(),
			Self::Sftp { loc, .. } => loc.file_name()?.as_strand(),
		})
	}

	#[inline]
	pub fn os_str(self) -> Cow<'a, OsStr> { self.components().os_str() }

	#[inline]
	pub fn pair(self) -> Option<(Self, PathDyn<'a>)> { Some((self.parent()?, self.urn())) }

	#[inline]
	pub fn pair2(self) -> Option<(Self, PathDyn<'a>)> {
		Some((self.parent()?, Some(self.entry_key()).filter(|k| !k.is_empty())?))
	}

	#[inline]
	pub fn entry_key(self) -> PathDyn<'a> {
		match self {
			Self::Hub { auth, .. } => PathDyn::Unix(typed_path::UnixPath::new(&auth.domain)),
			_ => self.urn(),
		}
	}

	pub fn parent(self) -> Option<Self> {
		let uri = self.uri();

		Some(match self {
			// Regular
			Self::Regular(loc) => Self::regular(loc.parent()?),

			// Search
			Self::Search { loc, .. } if uri.is_empty() => Self::regular(loc.parent()?),
			Self::Search { loc, auth } => {
				Self::Search { loc: Loc::new(loc.parent()?, loc.base(), loc.base()), auth }
			}

			// Mount
			Self::Mount { loc, .. } if uri.is_empty() => Self::regular(loc.parent()?),
			Self::Mount { loc, auth } if uri.components().nth(1).is_none() => {
				Self::Mount { loc: Loc::zeroed(loc.parent()?), auth }
			}
			Self::Mount { loc, auth } => {
				Self::Mount { loc: Loc::floated(loc.parent()?, loc.base()), auth }
			}

			// Hub
			Self::Hub { loc, auth } => {
				Self::Hub { loc: Loc::bare(loc.parent()?), auth: auth.parent.as_ref()? }
			}

			// Scope
			Self::Scope { loc, auth } => Self::Scope { loc: Loc::bare(loc.parent()?), auth },

			// SFTP
			Self::Sftp { loc, auth } => Self::Sftp { loc: Loc::bare(loc.parent()?), auth },
		})
	}

	#[inline]
	pub fn regular<T: AsRef<Path> + ?Sized>(path: &'a T) -> Self {
		Self::Regular(Loc::bare(path.as_ref()))
	}

	pub fn spec(self) -> Spec {
		let auth = match self {
			Self::Regular(_) => Auth::default_arc(),
			Self::Search { auth, .. }
			| Self::Mount { auth, .. }
			| Self::Hub { auth, .. }
			| Self::Scope { auth, .. }
			| Self::Sftp { auth, .. } => auth.clone(),
		};

		let (uri, urn) = Spec::retrieve_ports(self);
		Spec { auth, uri, urn }
	}

	#[inline]
	pub fn stem(self) -> Option<Strand<'a>> {
		Some(match self {
			Self::Regular(loc) => loc.file_stem()?.as_strand(),
			Self::Search { loc, .. } => loc.file_stem()?.as_strand(),
			Self::Mount { loc, .. } => loc.file_stem()?.as_strand(),
			Self::Hub { loc, .. } => loc.file_stem()?.as_strand(),
			Self::Scope { loc, .. } => loc.file_stem()?.as_strand(),
			Self::Sftp { loc, .. } => loc.file_stem()?.as_strand(),
		})
	}

	#[inline]
	pub fn to_owned(self) -> UrlBuf { self.into() }

	pub fn to_search(self, query: impl AsRef<str>) -> Result<UrlBuf, PathDynError> {
		Ok(UrlBuf::Search {
			loc:  LocBuf::<PathBuf>::zeroed(self.loc().to_os_owned()?),
			auth: Auth::search(query.as_ref()),
		})
	}

	pub fn trail(self) -> Self {
		let uri = self.uri();
		match self {
			Self::Regular(loc) => Self::Regular(Loc::bare(loc.trail())),

			Self::Search { loc, auth } if uri.is_empty() => {
				Self::Search { loc: Loc::zeroed(loc.trail()), auth }
			}
			Self::Search { loc, auth } => {
				Self::Search { loc: Loc::new(loc.trail(), loc.base(), loc.base()), auth }
			}

			Self::Mount { loc, auth } if uri.is_empty() => {
				Self::Mount { loc: Loc::zeroed(loc.trail()), auth }
			}
			Self::Mount { loc, auth } => {
				Self::Mount { loc: Loc::new(loc.trail(), loc.base(), loc.base()), auth }
			}

			Self::Hub { loc, auth } => Self::Hub {
				loc:  Loc::bare(loc.trail()),
				auth: auth.parent_at(loc.urn().dyn_path().components().auth_depth()),
			},

			Self::Scope { loc, auth } => Self::Scope { loc: Loc::bare(loc.trail()), auth },

			Self::Sftp { loc, auth } => Self::Sftp { loc: Loc::bare(loc.trail()), auth },
		}
	}

	pub fn triple(self) -> (PathDyn<'a>, PathDyn<'a>, PathDyn<'a>) {
		match self {
			Self::Regular(loc)
			| Self::Search { loc, .. }
			| Self::Mount { loc, .. }
			| Self::Hub { loc, .. } => {
				let (base, rest, urn) = loc.triple();
				(base.dyn_path(), rest.dyn_path(), urn.dyn_path())
			}
			Self::Scope { loc, .. } | Self::Sftp { loc, .. } => {
				let (base, rest, urn) = loc.triple();
				(base.dyn_path(), rest.dyn_path(), urn.dyn_path())
			}
		}
	}

	#[inline]
	pub fn try_ends_with(self, child: impl AsUrl) -> Result<bool, EndsWithError> {
		let child = child.as_url();
		Ok(self.loc().try_ends_with(child.loc())? && self.auth().covariant(child.auth()))
	}

	pub fn try_join(self, path: impl AsStrand) -> Result<UrlBuf, JoinError> {
		let path = path.as_strand();
		let joined = self.loc().try_join(path)?;

		Ok(match self {
			Self::Regular(_) => UrlBuf::Regular(joined.into_os()?.into()),

			Self::Search { loc, auth } if joined.try_starts_with(loc.base())? => UrlBuf::Search {
				loc:  LocBuf::<PathBuf>::new(joined.try_into()?, loc.base(), loc.base()),
				auth: auth.clone(),
			},
			Self::Search { auth, .. } => {
				UrlBuf::Search { loc: LocBuf::<PathBuf>::zeroed(joined.into_os()?), auth: auth.clone() }
			}

			Self::Mount { loc, auth } if joined.try_starts_with(loc.base())? => UrlBuf::Mount {
				loc:  LocBuf::<PathBuf>::floated(joined.try_into()?, loc.base()),
				auth: auth.clone(),
			},
			Self::Mount { auth, .. } => {
				UrlBuf::Mount { loc: LocBuf::<PathBuf>::zeroed(joined.into_os()?), auth: auth.clone() }
			}

			Self::Hub { auth, .. } => UrlBuf::Hub {
				loc:  joined.into_os()?.into(),
				auth: auth.clone().descend(path.as_os_path()?),
			},

			Self::Scope { auth, .. } => {
				UrlBuf::Scope { loc: joined.into_unix()?.into(), auth: auth.clone() }
			}

			Self::Sftp { auth, .. } => {
				UrlBuf::Sftp { loc: joined.into_unix()?.into(), auth: auth.clone() }
			}
		})
	}

	pub fn try_replace<'b>(self, take: usize, to: impl DynPathRef<'b>) -> Result<UrlCow<'b>> {
		self.try_replace_impl(take, to.dyn_path_ref())
	}

	fn try_replace_impl<'b>(self, take: usize, rep: PathDyn<'b>) -> Result<UrlCow<'b>> {
		let b = rep.encoded_bytes();
		if take == 0 {
			return UrlCow::try_from(b);
		} else if ParsedSpec::parse(b)?.has_scheme() {
			return UrlCow::try_from(b);
		}

		let loc = self.loc();
		let mut path = PathBufDyn::from_components(loc.kind(), loc.components().take(take - 1))?;
		path.try_push(rep)?;

		let url = match self {
			Self::Regular(_) => UrlBuf::from(path.into_os()?),

			Self::Search { loc, auth } if path.try_starts_with(loc.trail())? => UrlBuf::Search {
				loc:  LocBuf::<PathBuf>::new(path.into_os()?, loc.base(), loc.trail()),
				auth: auth.clone(),
			},
			Self::Mount { loc, auth } if path.try_starts_with(loc.trail())? => UrlBuf::Mount {
				loc:  LocBuf::<std::path::PathBuf>::new(path.into_os()?, loc.base(), loc.trail()),
				auth: auth.clone(),
			},
			Self::Hub { loc, auth } if path.try_starts_with(loc.trail())? => UrlBuf::Hub {
				auth: auth.clone().with_parent_depth(path.components().auth_depth()),
				loc:  LocBuf::<PathBuf>::new(path.into_os()?, loc.base(), loc.trail()),
			},
			Self::Scope { loc, auth } if path.try_starts_with(loc.trail())? => UrlBuf::Scope {
				loc:  LocBuf::<typed_path::UnixPathBuf>::new(path.into_unix()?, loc.base(), loc.trail()),
				auth: auth.clone(),
			},
			Self::Sftp { loc, auth } if path.try_starts_with(loc.trail())? => UrlBuf::Sftp {
				loc:  LocBuf::<typed_path::UnixPathBuf>::new(path.into_unix()?, loc.base(), loc.trail()),
				auth: auth.clone(),
			},

			Self::Search { auth, .. } => UrlBuf::Search {
				loc:  LocBuf::<std::path::PathBuf>::saturated(path.into_os()?, self.kind()),
				auth: auth.clone(),
			},
			Self::Mount { auth, .. } => UrlBuf::Mount {
				loc:  LocBuf::<std::path::PathBuf>::saturated(path.into_os()?, self.kind()),
				auth: auth.clone(),
			},
			Self::Hub { auth, .. } => UrlBuf::Hub {
				auth: auth.clone().with_parent_depth(path.components().auth_depth()),
				loc:  LocBuf::<PathBuf>::saturated(path.into_os()?, self.kind()),
			},
			Self::Scope { auth, .. } => UrlBuf::Scope {
				loc:  LocBuf::<typed_path::UnixPathBuf>::saturated(path.into_unix()?, self.kind()),
				auth: auth.clone(),
			},
			Self::Sftp { auth, .. } => UrlBuf::Sftp {
				loc:  LocBuf::<typed_path::UnixPathBuf>::saturated(path.into_unix()?, self.kind()),
				auth: auth.clone(),
			},
		};

		Ok(url.into())
	}

	pub fn try_starts_with(self, base: impl AsUrl) -> Result<bool, StartsWithError> {
		let base = base.as_url();
		Ok(
			self.loc().try_starts_with(base.loc())?
				&& self.auth_at(base).is_some_and(|a| a == base.auth()),
		)
	}

	pub fn try_strip_prefix(self, base: impl AsUrl) -> Result<PathDyn<'a>, StripPrefixError> {
		use StripPrefixError::{Exotic, NotPrefix};
		use Url as U;

		let base = base.as_url();
		let prefix = self.loc().try_strip_prefix(base.loc())?;
		if self.auth_at(base).is_some_and(|a| a == base.auth()) {
			return Ok(prefix);
		}

		match (self, base) {
			// A mount portal is the local source file until it gains an inner URI.
			(U::Regular(_) | U::Search { .. }, U::Mount { .. }) => {
				base.uri().is_empty().then_some(prefix).ok_or(NotPrefix)
			}
			(U::Mount { .. }, U::Regular(_) | U::Search { .. }) => {
				self.uri().is_empty().then_some(prefix).ok_or(NotPrefix)
			}
			_ => Err(Exotic),
		}
	}

	pub fn try_strip_suffix(self, other: impl AsUrl) -> Result<PathDyn<'a>, StripSuffixError> {
		use StripSuffixError::{Exotic, NotSuffix};
		use Url as U;

		let other = other.as_url();
		let suffix = self.loc().try_strip_suffix(other.loc())?;
		if self.auth().covariant(other.auth()) {
			return Ok(suffix);
		}

		match (self, other) {
			// A mount portal is the local source file until it gains an inner URI.
			(U::Regular(_) | U::Search { .. }, U::Mount { .. }) => {
				other.uri().is_empty().then_some(suffix).ok_or(NotSuffix)
			}
			(U::Mount { .. }, U::Regular(_) | U::Search { .. }) => {
				self.uri().is_empty().then_some(suffix).ok_or(NotSuffix)
			}
			_ => Err(Exotic),
		}
	}

	#[inline]
	pub fn uri(self) -> PathDyn<'a> {
		match self {
			Self::Regular(loc) => loc.uri().dyn_path(),
			Self::Search { loc, .. } => loc.uri().dyn_path(),
			Self::Mount { loc, .. } => loc.uri().dyn_path(),
			Self::Hub { loc, .. } => loc.uri().dyn_path(),
			Self::Scope { loc, .. } => loc.uri().dyn_path(),
			Self::Sftp { loc, .. } => loc.uri().dyn_path(),
		}
	}

	#[inline]
	pub fn urn(self) -> PathDyn<'a> {
		match self {
			Self::Regular(loc) => loc.urn().dyn_path(),
			Self::Search { loc, .. } => loc.urn().dyn_path(),
			Self::Mount { loc, .. } => loc.urn().dyn_path(),
			Self::Hub { loc, .. } => loc.urn().dyn_path(),
			Self::Scope { loc, .. } => loc.urn().dyn_path(),
			Self::Sftp { loc, .. } => loc.urn().dyn_path(),
		}
	}
}
