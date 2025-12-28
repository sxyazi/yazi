use std::{borrow::Cow, ffi::OsStr, fmt::{Debug, Formatter}, path::{Path, PathBuf}};

use anyhow::Result;
use hashbrown::Equivalent;
use serde::Serialize;

use super::Encode as EncodeUrl;
use crate::{loc::{Loc, LocBuf}, path::{AsPath, AsPathRef, EndsWithError, JoinError, PathBufDyn, PathDyn, PathDynError, PathLike, StartsWithError, StripPrefixError, StripSuffixError}, pool::InternStr, scheme::{Encode as EncodeScheme, SchemeCow, SchemeKind, SchemeRef}, strand::{AsStrand, Strand}, url::{AsUrl, Components, UrlBuf, UrlCow}};

#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub enum Url<'a> {
	Regular(Loc<'a>),
	Search { loc: Loc<'a>, domain: &'a str },
	Archive { loc: Loc<'a>, domain: &'a str },
	Sftp { loc: Loc<'a, &'a typed_path::UnixPath>, domain: &'a str },
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
			write!(f, "{}{}", EncodeScheme(*self), self.loc().display())
		}
	}
}

impl Serialize for Url<'_> {
	fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
		let (kind, loc) = (self.kind(), self.loc());
		match (kind == SchemeKind::Regular, loc.to_str()) {
			(true, Ok(s)) => serializer.serialize_str(s),
			(false, Ok(s)) => serializer.serialize_str(&format!("{}{s}", EncodeScheme(*self))),
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
	pub fn as_regular(self) -> Result<Self, PathDynError> {
		Ok(Self::Regular(Loc::bare(self.loc().as_os()?)))
	}

	pub fn base(self) -> Self {
		match self {
			Self::Regular(loc) => Self::Regular(Loc::bare(loc.base())),
			Self::Search { loc, domain } => Self::Search { loc: Loc::zeroed(loc.base()), domain },
			Self::Archive { loc, domain } => Self::Archive { loc: Loc::zeroed(loc.base()), domain },
			Self::Sftp { loc, domain } => Self::Sftp { loc: Loc::bare(loc.base()), domain },
		}
	}

	#[inline]
	pub fn components(self) -> Components<'a> { Components::from(self) }

	#[inline]
	pub fn covariant(self, other: impl AsUrl) -> bool {
		let other = other.as_url();
		self.loc() == other.loc() && self.scheme().covariant(other.scheme())
	}

	#[inline]
	pub fn ext(self) -> Option<Strand<'a>> {
		Some(match self {
			Self::Regular(loc) => loc.extension()?.as_strand(),
			Self::Search { loc, .. } => loc.extension()?.as_strand(),
			Self::Archive { loc, .. } => loc.extension()?.as_strand(),
			Self::Sftp { loc, .. } => loc.extension()?.as_strand(),
		})
	}

	#[inline]
	pub fn has_base(self) -> bool {
		match self {
			Self::Regular(loc) => loc.has_base(),
			Self::Search { loc, .. } => loc.has_base(),
			Self::Archive { loc, .. } => loc.has_base(),
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
			Self::Archive { loc, .. } => loc.has_trail(),
			Self::Sftp { loc, .. } => loc.has_trail(),
		}
	}

	#[inline]
	pub fn is_absolute(self) -> bool { self.loc().is_absolute() }

	#[inline]
	pub fn is_archive(self) -> bool { matches!(self, Self::Archive { .. }) }

	#[inline]
	pub fn is_internal(self) -> bool {
		match self {
			Self::Regular(_) | Self::Sftp { .. } => true,
			Self::Search { .. } => !self.uri().is_empty(),
			Self::Archive { .. } => false,
		}
	}

	#[inline]
	pub fn is_regular(self) -> bool { matches!(self, Self::Regular(_)) }

	#[inline]
	pub fn is_search(self) -> bool { matches!(self, Self::Search { .. }) }

	#[inline]
	pub fn kind(self) -> SchemeKind {
		match self {
			Self::Regular(_) => SchemeKind::Regular,
			Self::Search { .. } => SchemeKind::Search,
			Self::Archive { .. } => SchemeKind::Archive,
			Self::Sftp { .. } => SchemeKind::Sftp,
		}
	}

	#[inline]
	pub fn loc(self) -> PathDyn<'a> {
		match self {
			Self::Regular(loc) => loc.as_path(),
			Self::Search { loc, .. } => loc.as_path(),
			Self::Archive { loc, .. } => loc.as_path(),
			Self::Sftp { loc, .. } => loc.as_path(),
		}
	}

	#[inline]
	pub fn name(self) -> Option<Strand<'a>> {
		Some(match self {
			Self::Regular(loc) => loc.file_name()?.as_strand(),
			Self::Search { loc, .. } => loc.file_name()?.as_strand(),
			Self::Archive { loc, .. } => loc.file_name()?.as_strand(),
			Self::Sftp { loc, .. } => loc.file_name()?.as_strand(),
		})
	}

	#[inline]
	pub fn os_str(self) -> Cow<'a, OsStr> { self.components().os_str() }

	#[inline]
	pub fn pair(self) -> Option<(Self, PathDyn<'a>)> { Some((self.parent()?, self.urn())) }

	pub fn parent(self) -> Option<Self> {
		let uri = self.uri();

		Some(match self {
			// Regular
			Self::Regular(loc) => Self::regular(loc.parent()?),

			// Search
			Self::Search { loc, .. } if uri.is_empty() => Self::regular(loc.parent()?),
			Self::Search { loc, domain } => {
				Self::Search { loc: Loc::new(loc.parent()?, loc.base(), loc.base()), domain }
			}

			// Archive
			Self::Archive { loc, .. } if uri.is_empty() => Self::regular(loc.parent()?),
			Self::Archive { loc, domain } if uri.components().nth(1).is_none() => {
				Self::Archive { loc: Loc::zeroed(loc.parent()?), domain }
			}
			Self::Archive { loc, domain } => {
				Self::Archive { loc: Loc::floated(loc.parent()?, loc.base()), domain }
			}

			// SFTP
			Self::Sftp { loc, domain } => Self::Sftp { loc: Loc::bare(loc.parent()?), domain },
		})
	}

	#[inline]
	pub fn regular<T: AsRef<Path> + ?Sized>(path: &'a T) -> Self {
		Self::Regular(Loc::bare(path.as_ref()))
	}

	#[inline]
	pub fn scheme(self) -> SchemeRef<'a> {
		let (uri, urn) = SchemeCow::retrieve_ports(self);
		match self {
			Self::Regular(_) => SchemeRef::Regular { uri, urn },
			Self::Search { domain, .. } => SchemeRef::Search { domain, uri, urn },
			Self::Archive { domain, .. } => SchemeRef::Archive { domain, uri, urn },
			Self::Sftp { domain, .. } => SchemeRef::Sftp { domain, uri, urn },
		}
	}

	#[inline]
	pub fn stem(self) -> Option<Strand<'a>> {
		Some(match self {
			Self::Regular(loc) => loc.file_stem()?.as_strand(),
			Self::Search { loc, .. } => loc.file_stem()?.as_strand(),
			Self::Archive { loc, .. } => loc.file_stem()?.as_strand(),
			Self::Sftp { loc, .. } => loc.file_stem()?.as_strand(),
		})
	}

	#[inline]
	pub fn to_owned(self) -> UrlBuf { self.into() }

	pub fn trail(self) -> Self {
		let uri = self.uri();
		match self {
			Self::Regular(loc) => Self::Regular(Loc::bare(loc.trail())),

			Self::Search { loc, domain } if uri.is_empty() => {
				Self::Search { loc: Loc::zeroed(loc.trail()), domain }
			}
			Self::Search { loc, domain } => {
				Self::Search { loc: Loc::new(loc.trail(), loc.base(), loc.base()), domain }
			}

			Self::Archive { loc, domain } if uri.is_empty() => {
				Self::Archive { loc: Loc::zeroed(loc.trail()), domain }
			}
			Self::Archive { loc, domain } => {
				Self::Archive { loc: Loc::new(loc.trail(), loc.base(), loc.base()), domain }
			}

			Self::Sftp { loc, domain } => Self::Sftp { loc: Loc::bare(loc.trail()), domain },
		}
	}

	pub fn triple(self) -> (PathDyn<'a>, PathDyn<'a>, PathDyn<'a>) {
		match self {
			Self::Regular(loc) | Self::Search { loc, .. } | Self::Archive { loc, .. } => {
				let (base, rest, urn) = loc.triple();
				(base.as_path(), rest.as_path(), urn.as_path())
			}
			Self::Sftp { loc, .. } => {
				let (base, rest, urn) = loc.triple();
				(base.as_path(), rest.as_path(), urn.as_path())
			}
		}
	}

	#[inline]
	pub fn try_ends_with(self, child: impl AsUrl) -> Result<bool, EndsWithError> {
		let child = child.as_url();
		Ok(self.loc().try_ends_with(child.loc())? && self.scheme().covariant(child.scheme()))
	}

	pub fn try_join(self, path: impl AsStrand) -> Result<UrlBuf, JoinError> {
		let joined = self.loc().try_join(path)?;

		Ok(match self {
			Self::Regular(_) => UrlBuf::Regular(joined.into_os()?.into()),

			Self::Search { loc, domain } if joined.try_starts_with(loc.base())? => UrlBuf::Search {
				loc:    LocBuf::<PathBuf>::new(joined.try_into()?, loc.base(), loc.base()),
				domain: domain.intern(),
			},
			Self::Search { domain, .. } => UrlBuf::Search {
				loc:    LocBuf::<PathBuf>::zeroed(joined.into_os()?),
				domain: domain.intern(),
			},

			Self::Archive { loc, domain } if joined.try_starts_with(loc.base())? => UrlBuf::Archive {
				loc:    LocBuf::<PathBuf>::floated(joined.try_into()?, loc.base()),
				domain: domain.intern(),
			},
			Self::Archive { domain, .. } => UrlBuf::Archive {
				loc:    LocBuf::<PathBuf>::zeroed(joined.into_os()?),
				domain: domain.intern(),
			},

			Self::Sftp { domain, .. } => {
				UrlBuf::Sftp { loc: joined.into_unix()?.into(), domain: domain.intern() }
			}
		})
	}

	pub fn try_replace<'b>(self, take: usize, to: impl AsPathRef<'b>) -> Result<UrlCow<'b>> {
		self.try_replace_impl(take, to.as_path_ref())
	}

	fn try_replace_impl<'b>(self, take: usize, rep: PathDyn<'b>) -> Result<UrlCow<'b>> {
		let b = rep.encoded_bytes();
		if take == 0 {
			return UrlCow::try_from(b);
		} else if SchemeKind::parse(b)?.is_some() {
			return UrlCow::try_from(b);
		}

		let loc = self.loc();
		let mut path = PathBufDyn::from_components(loc.kind(), loc.components().take(take - 1))?;
		path.try_push(rep)?;

		let url = match self {
			Self::Regular(_) => UrlBuf::from(path.into_os()?),

			Self::Search { loc, domain } if path.try_starts_with(loc.trail())? => UrlBuf::Search {
				loc:    LocBuf::<PathBuf>::new(path.into_os()?, loc.base(), loc.trail()),
				domain: domain.intern(),
			},
			Self::Archive { loc, domain } if path.try_starts_with(loc.trail())? => UrlBuf::Archive {
				loc:    LocBuf::<std::path::PathBuf>::new(path.into_os()?, loc.base(), loc.trail()),
				domain: domain.intern(),
			},
			Self::Sftp { loc, domain } if path.try_starts_with(loc.trail())? => UrlBuf::Sftp {
				loc:    LocBuf::<typed_path::UnixPathBuf>::new(path.into_unix()?, loc.base(), loc.trail()),
				domain: domain.intern(),
			},

			Self::Search { domain, .. } => UrlBuf::Search {
				loc:    LocBuf::<std::path::PathBuf>::saturated(path.into_os()?, self.kind()),
				domain: domain.intern(),
			},
			Self::Archive { domain, .. } => UrlBuf::Archive {
				loc:    LocBuf::<std::path::PathBuf>::saturated(path.into_os()?, self.kind()),
				domain: domain.intern(),
			},
			Self::Sftp { domain, .. } => UrlBuf::Sftp {
				loc:    LocBuf::<typed_path::UnixPathBuf>::saturated(path.into_unix()?, self.kind()),
				domain: domain.intern(),
			},
		};

		Ok(url.into())
	}

	#[inline]
	pub fn try_starts_with(self, base: impl AsUrl) -> Result<bool, StartsWithError> {
		let base = base.as_url();
		Ok(self.loc().try_starts_with(base.loc())? && self.scheme().covariant(base.scheme()))
	}

	pub fn try_strip_prefix(self, base: impl AsUrl) -> Result<PathDyn<'a>, StripPrefixError> {
		use StripPrefixError::{Exotic, NotPrefix};
		use Url as U;

		let base = base.as_url();
		let prefix = self.loc().try_strip_prefix(base.loc())?;

		match (self, base) {
			// Same scheme
			(U::Regular(_), U::Regular(_)) => Ok(prefix),
			(U::Search { .. }, U::Search { .. }) => Ok(prefix),
			(U::Archive { domain: a, .. }, U::Archive { domain: b, .. }) => {
				Some(prefix).filter(|_| a == b).ok_or(Exotic)
			}
			(U::Sftp { domain: a, .. }, U::Sftp { domain: b, .. }) => {
				Some(prefix).filter(|_| a == b).ok_or(Exotic)
			}

			// Both are local files
			(U::Regular(_), U::Search { .. }) => Ok(prefix),
			(U::Search { .. }, U::Regular(_)) => Ok(prefix),

			// Only the entry of archives is a local file
			(U::Regular(_), U::Archive { .. }) => {
				Some(prefix).filter(|_| base.uri().is_empty()).ok_or(NotPrefix)
			}
			(U::Search { .. }, U::Archive { .. }) => {
				Some(prefix).filter(|_| base.uri().is_empty()).ok_or(NotPrefix)
			}
			(U::Archive { .. }, U::Regular(_)) => {
				Some(prefix).filter(|_| self.uri().is_empty()).ok_or(NotPrefix)
			}
			(U::Archive { .. }, U::Search { .. }) => {
				Some(prefix).filter(|_| self.uri().is_empty()).ok_or(NotPrefix)
			}

			// Independent virtual file space
			(U::Regular(_), U::Sftp { .. }) => Err(Exotic),
			(U::Search { .. }, U::Sftp { .. }) => Err(Exotic),
			(U::Archive { .. }, U::Sftp { .. }) => Err(Exotic),
			(U::Sftp { .. }, U::Regular(_)) => Err(Exotic),
			(U::Sftp { .. }, U::Search { .. }) => Err(Exotic),
			(U::Sftp { .. }, U::Archive { .. }) => Err(Exotic),
		}
	}

	pub fn try_strip_suffix(self, other: impl AsUrl) -> Result<PathDyn<'a>, StripSuffixError> {
		use StripSuffixError::{Exotic, NotSuffix};
		use Url as U;

		let other = other.as_url();
		let suffix = self.loc().try_strip_suffix(other.loc())?;

		match (self, other) {
			// Same scheme
			(U::Regular(_), U::Regular(_)) => Ok(suffix),
			(U::Search { .. }, U::Search { .. }) => Ok(suffix),
			(U::Archive { domain: a, .. }, U::Archive { domain: b, .. }) => {
				Some(suffix).filter(|_| a == b).ok_or(Exotic)
			}
			(U::Sftp { domain: a, .. }, U::Sftp { domain: b, .. }) => {
				Some(suffix).filter(|_| a == b).ok_or(Exotic)
			}

			// Both are local files
			(U::Regular(_), U::Search { .. }) => Ok(suffix),
			(U::Search { .. }, U::Regular(_)) => Ok(suffix),

			// Only the entry of archives is a local file
			(U::Regular(_), U::Archive { .. }) => {
				Some(suffix).filter(|_| other.uri().is_empty()).ok_or(NotSuffix)
			}
			(U::Search { .. }, U::Archive { .. }) => {
				Some(suffix).filter(|_| other.uri().is_empty()).ok_or(NotSuffix)
			}
			(U::Archive { .. }, U::Regular(_)) => {
				Some(suffix).filter(|_| self.uri().is_empty()).ok_or(NotSuffix)
			}
			(U::Archive { .. }, U::Search { .. }) => {
				Some(suffix).filter(|_| self.uri().is_empty()).ok_or(NotSuffix)
			}

			// Independent virtual file space
			(U::Regular(_), U::Sftp { .. }) => Err(Exotic),
			(U::Search { .. }, U::Sftp { .. }) => Err(Exotic),
			(U::Archive { .. }, U::Sftp { .. }) => Err(Exotic),
			(U::Sftp { .. }, U::Regular(_)) => Err(Exotic),
			(U::Sftp { .. }, U::Search { .. }) => Err(Exotic),
			(U::Sftp { .. }, U::Archive { .. }) => Err(Exotic),
		}
	}

	#[inline]
	pub fn uri(self) -> PathDyn<'a> {
		match self {
			Self::Regular(loc) => loc.uri().as_path(),
			Self::Search { loc, .. } => loc.uri().as_path(),
			Self::Archive { loc, .. } => loc.uri().as_path(),
			Self::Sftp { loc, .. } => loc.uri().as_path(),
		}
	}

	#[inline]
	pub fn urn(self) -> PathDyn<'a> {
		match self {
			Self::Regular(loc) => loc.urn().as_path(),
			Self::Search { loc, .. } => loc.urn().as_path(),
			Self::Archive { loc, .. } => loc.urn().as_path(),
			Self::Sftp { loc, .. } => loc.urn().as_path(),
		}
	}
}
