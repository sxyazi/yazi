use std::{borrow::Cow, ffi::{OsStr, OsString}, fmt::{Debug, Display, Formatter}, hash::{BuildHasher, Hash, Hasher}, ops::Deref, path::{Path, PathBuf}};

use percent_encoding::{AsciiSet, CONTROLS, percent_decode, percent_encode};
use serde::{Deserialize, Serialize};

use super::UrnBuf;
use crate::{BytesExt, IntoOsStr, url::{Loc, Scheme}};

const ENCODE_SET: &AsciiSet = &CONTROLS.add(b'#');

#[derive(Clone, Default, Eq, Ord, PartialOrd)]
pub struct Url {
	loc:        Loc,
	pub scheme: Scheme,
	pub frag:   OsString,
}

impl Deref for Url {
	type Target = Loc;

	fn deref(&self) -> &Self::Target { &self.loc }
}

// FIXME: remove
impl Debug for Url {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		let Self { scheme, loc, frag } = self;
		match scheme {
			Scheme::Regular => write!(f, "{scheme}{}", loc.display()),
			Scheme::Search => write!(f, "{scheme}{}#{}", loc.display(), frag.display()),
			Scheme::SearchItem => write!(f, "{scheme}{}", loc.display()),
			Scheme::Archive => write!(f, "{scheme}{}", loc.display()),
			Scheme::Sftp(_) => write!(f, "{scheme}{}", loc.display()),
		}
	}
}

impl From<Loc> for Url {
	fn from(loc: Loc) -> Self { Self { loc, scheme: Scheme::Regular, frag: OsString::new() } }
}

impl From<PathBuf> for Url {
	fn from(path: PathBuf) -> Self { Loc::from(path).into() }
}

impl From<&PathBuf> for Url {
	fn from(path: &PathBuf) -> Self { path.to_owned().into() }
}

impl From<&Path> for Url {
	fn from(path: &Path) -> Self { path.to_path_buf().into() }
}

impl TryFrom<&[u8]> for Url {
	type Error = anyhow::Error;

	fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
		let (scheme, skip) = Scheme::parse(bytes)?;
		let rest = &bytes[skip..];

		if scheme == Scheme::Regular {
			return Ok(Self { loc: rest.into_os_str()?.into(), scheme, frag: OsString::new() });
		}

		let (loc, frag) = match rest.split_by_seq(b"#") {
			None => (rest, OsString::new()),
			Some((a, b)) => (a, b.into_os_str()?.into_owned()),
		};

		Ok(Url { loc: Cow::from(percent_decode(loc)).into_os_str()?.into(), scheme, frag })
	}
}

impl TryFrom<&str> for Url {
	type Error = anyhow::Error;

	fn try_from(value: &str) -> Result<Self, Self::Error> { value.as_bytes().try_into() }
}

impl TryFrom<String> for Url {
	type Error = anyhow::Error;

	fn try_from(value: String) -> Result<Self, Self::Error> { value.as_bytes().try_into() }
}

impl AsRef<Url> for Url {
	fn as_ref(&self) -> &Url { self }
}

// FIXME: remove
impl AsRef<Path> for Url {
	fn as_ref(&self) -> &Path { &self.loc }
}

// FIXME: remove
impl Display for Url {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		if matches!(self.scheme, Scheme::Regular | Scheme::SearchItem) {
			return write!(f, "{}", self.loc.display());
		}

		let loc = percent_encode(self.loc.as_os_str().as_encoded_bytes(), ENCODE_SET);
		write!(f, "{}{loc}", self.scheme)?;

		if !self.frag.is_empty() {
			write!(f, "#{}", percent_encode(self.frag.as_encoded_bytes(), ENCODE_SET))?;
		}

		Ok(())
	}
}

// FIXME: remove
impl From<&Url> for String {
	fn from(url: &Url) -> Self { url.to_string() }
}

impl<'a> From<&'a Url> for Cow<'a, Url> {
	fn from(url: &'a Url) -> Self { Cow::Borrowed(url) }
}

impl From<Url> for Cow<'_, Url> {
	fn from(url: Url) -> Self { Cow::Owned(url) }
}

impl From<Cow<'_, Url>> for Url {
	fn from(url: Cow<'_, Url>) -> Self { url.into_owned() }
}

impl Url {
	#[inline]
	pub fn base(&self) -> Url {
		let loc: Loc = self.loc.base().into();
		match &self.scheme {
			Scheme::Regular => Self { loc, scheme: Scheme::Regular, frag: OsString::new() },
			Scheme::Search => Self { loc, scheme: Scheme::Search, frag: OsString::new() },
			Scheme::SearchItem => Self { loc, scheme: Scheme::Search, frag: OsString::new() },
			Scheme::Archive => Self { loc, scheme: Scheme::Archive, frag: OsString::new() },
			Scheme::Sftp(_) => Self { loc, scheme: self.scheme.clone(), frag: OsString::new() },
		}
	}

	pub fn join(&self, path: impl AsRef<Path>) -> Self {
		match self.scheme {
			Scheme::Regular => Self {
				loc:    self.loc.join(path).into(),
				scheme: Scheme::Regular,
				frag:   OsString::new(),
			},
			Scheme::Search => Self {
				loc:    Loc::with(&self.loc, self.loc.join(path)),
				scheme: Scheme::SearchItem,
				frag:   OsString::new(),
			},
			Scheme::SearchItem => Self {
				loc:    Loc::with(self.loc.base(), self.loc.join(path)),
				scheme: Scheme::SearchItem,
				frag:   OsString::new(),
			},
			Scheme::Archive => Self {
				loc:    self.loc.join(path).into(),
				scheme: Scheme::Archive,
				frag:   OsString::new(),
			},
			Scheme::Sftp(_) => Self {
				loc:    self.loc.join(path).into(),
				scheme: self.scheme.clone(),
				frag:   OsString::new(),
			},
		}
	}

	pub fn parent_url(&self) -> Option<Url> {
		let parent = self.loc.parent()?;
		let base = self.loc.base();

		Some(match &self.scheme {
			Scheme::Regular => {
				Self { loc: parent.into(), scheme: Scheme::Regular, frag: OsString::new() }
			}
			Scheme::Search => {
				Self { loc: parent.into(), scheme: Scheme::Regular, frag: OsString::new() }
			}
			Scheme::SearchItem if parent == base => {
				Self { loc: parent.into(), scheme: Scheme::Search, frag: OsString::new() }
			}
			Scheme::SearchItem => Self {
				loc:    Loc::with(base, parent.to_owned()),
				scheme: Scheme::SearchItem,
				frag:   OsString::new(),
			},
			Scheme::Archive => {
				Self { loc: parent.into(), scheme: Scheme::Regular, frag: OsString::new() }
			}
			Scheme::Sftp(_) => {
				Self { loc: parent.into(), scheme: self.scheme.clone(), frag: OsString::new() }
			}
		})
	}

	#[inline]
	pub fn as_path(&self) -> Option<&Path> {
		match &self.scheme {
			Scheme::Regular | Scheme::Search | Scheme::SearchItem => Some(&self.loc),
			Scheme::Archive | Scheme::Sftp(_) => None,
		}
	}

	#[inline]
	pub fn set_name(&mut self, name: impl AsRef<OsStr>) { self.loc.set_name(name); }

	#[inline]
	pub fn pair(&self) -> Option<(Self, UrnBuf)> { Some((self.parent_url()?, self.loc.urn_owned())) }

	#[inline]
	pub fn hash_u64(&self) -> u64 { foldhash::fast::FixedState::default().hash_one(self) }

	#[inline]
	pub fn rebase(&self, parent: &Path) -> Self {
		debug_assert!(self.is_regular());
		self.loc.rebase(parent).into()
	}
}

impl Url {
	// --- Regular
	#[inline]
	pub fn is_regular(&self) -> bool { self.scheme == Scheme::Regular }

	#[inline]
	pub fn to_regular(&self) -> Self {
		Self { loc: self.loc.clone(), scheme: Scheme::Regular, frag: OsString::new() }
	}

	#[inline]
	pub fn into_regular(mut self) -> Self {
		self.scheme = Scheme::Regular;
		self.frag = OsString::new();
		self
	}

	// --- Search
	#[inline]
	pub fn is_search(&self) -> bool { self.scheme == Scheme::Search }

	#[inline]
	pub fn to_search(&self, frag: impl AsRef<OsStr>) -> Self {
		Self { loc: self.loc.clone(), scheme: Scheme::Search, frag: frag.as_ref().to_owned() }
	}

	#[inline]
	pub fn into_search(mut self, frag: impl AsRef<OsStr>) -> Self {
		self.scheme = Scheme::Search;
		self.frag = frag.as_ref().to_owned();
		self
	}

	// --- Archive
	#[inline]
	pub fn is_archive(&self) -> bool { self.scheme == Scheme::Archive }

	// FIXME: remove
	#[inline]
	pub fn into_path(self) -> PathBuf { self.loc.into_path() }
}

impl Hash for Url {
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.loc.hash(state);

		match &self.scheme {
			Scheme::Regular => {}
			Scheme::Search => {
				self.scheme.hash(state);
				self.frag.hash(state);
			}
			Scheme::SearchItem => {}
			Scheme::Archive => {
				self.scheme.hash(state);
			}
			Scheme::Sftp(_) => {
				self.scheme.hash(state);
			}
		}
	}
}

impl PartialEq for Url {
	fn eq(&self, other: &Self) -> bool {
		if self.loc != other.loc {
			return false;
		}

		match (&self.scheme, &other.scheme) {
			(Scheme::Regular | Scheme::SearchItem, Scheme::Regular | Scheme::SearchItem) => true,
			(Scheme::Search, _) | (_, Scheme::Search) => {
				self.scheme == other.scheme && self.frag == other.frag
			}
			_ => self.scheme == other.scheme,
		}
	}
}

impl Serialize for Url {
	fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
		serializer.collect_str(self)
	}
}

impl<'de> Deserialize<'de> for Url {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		let s = String::deserialize(deserializer)?;
		Url::try_from(s).map_err(serde::de::Error::custom)
	}
}
