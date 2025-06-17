use std::{borrow::Cow, ffi::{OsStr, OsString}, fmt::{Debug, Display, Formatter}, hash::{BuildHasher, Hash, Hasher}, ops::Deref, path::{Path, PathBuf}};

use percent_encoding::{AsciiSet, CONTROLS, percent_decode, percent_encode};
use serde::{Deserialize, Serialize};

use super::UrnBuf;
use crate::{BytesExt, IntoOsStr, url::{Loc, Scheme}};

const ENCODE_SET: &AsciiSet = &CONTROLS.add(b'#');

#[derive(Clone, Default, Eq, Ord, PartialOrd)]
pub struct Url {
	loc:    Loc,
	scheme: Scheme,
	frag:   OsString,
}

impl Deref for Url {
	type Target = Loc;

	fn deref(&self) -> &Self::Target { &self.loc }
}

impl Debug for Url {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		match self.scheme {
			Scheme::Regular => write!(f, "Regular({:?})", self.loc),
			Scheme::Search => write!(f, "Search({:?}, {})", self.loc, self.frag.display()),
			Scheme::SearchItem => write!(f, "SearchItem({:?})", self.loc),
			Scheme::Archive => write!(f, "Archive({:?})", self.loc),
		}
	}
}

impl From<Loc> for Url {
	fn from(loc: Loc) -> Self { Self { loc, ..Default::default() } }
}

impl From<PathBuf> for Url {
	fn from(path: PathBuf) -> Self { Loc::new(path).into() }
}

impl From<&PathBuf> for Url {
	fn from(path: &PathBuf) -> Self { path.to_owned().into() }
}

impl From<&Path> for Url {
	fn from(path: &Path) -> Self { path.to_owned().into() }
}

impl TryFrom<&[u8]> for Url {
	type Error = anyhow::Error;

	fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
		let mut url = Url::default();
		let Some((scheme, rest)) = value.split_by_seq(b"://") else {
			url.loc = Loc::new(value.into_os_str()?.into_owned().into());
			return Ok(url);
		};

		url.scheme = Scheme::try_from(scheme)?;
		if url.scheme == Scheme::Regular {
			url.loc = Loc::new(rest.into_os_str()?.into_owned().into());
			return Ok(url);
		}

		let (loc, frag) = match rest.split_by_seq(b"#") {
			None => (rest, OsString::new()),
			Some((a, b)) => (a, b.into_os_str()?.into_owned()),
		};

		// FIXME: use `Loc::from(base, path)` instead
		url.loc = Loc::new(Cow::from(percent_decode(loc)).into_os_str()?.into_owned().into());
		url.frag = frag;

		Ok(url)
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

impl AsRef<Path> for Url {
	fn as_ref(&self) -> &Path { &self.loc }
}

impl AsRef<OsStr> for Url {
	fn as_ref(&self) -> &OsStr { self.loc.as_os_str() }
}

impl Display for Url {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		if matches!(self.scheme, Scheme::Regular | Scheme::SearchItem) {
			return write!(f, "{}", self.loc.display());
		}

		let loc = percent_encode(self.loc.as_os_str().as_encoded_bytes(), ENCODE_SET);
		write!(f, "{}://{loc}", self.scheme)?;

		if !self.frag.is_empty() {
			write!(f, "#{}", percent_encode(self.frag.as_encoded_bytes(), ENCODE_SET))?;
		}

		Ok(())
	}
}

impl From<&Url> for String {
	fn from(url: &Url) -> Self { url.to_string() }
}

impl Url {
	#[inline]
	pub fn join(&self, path: impl AsRef<Path>) -> Self {
		match self.scheme {
			Scheme::Regular => Self::from(self.loc.join(path)),
			Scheme::Search => {
				let loc = Loc::from(&self.loc, self.loc.join(path));
				Self::from(loc).into_search_item()
			}
			Scheme::SearchItem => {
				let loc = Loc::from(self.loc.base(), self.loc.join(path));
				Self::from(loc).into_search_item()
			}
			Scheme::Archive => Self::from(self.loc.join(path)).into_archive(),
		}
	}

	#[inline]
	pub fn parent_url(&self) -> Option<Url> {
		let p = self.loc.parent()?;
		Some(match self.scheme {
			Scheme::Regular | Scheme::Search => Self::from(p),
			Scheme::SearchItem => {
				if p == self.loc.base() {
					Self::from(p).into_search("")
				} else {
					Self::from(p).into_search_item()
				}
			}
			Scheme::Archive => Self::from(p),
		})
	}

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
	pub fn to_regular(&self) -> Self { Self { loc: self.loc.clone(), ..Default::default() } }

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

	#[inline]
	pub fn is_search_item(&self) -> bool { self.scheme == Scheme::SearchItem }

	#[inline]
	pub fn into_search_item(mut self) -> Self {
		self.scheme = Scheme::SearchItem;
		self.frag = OsString::new();
		self
	}

	#[inline]
	pub fn is_archive(&self) -> bool { self.scheme == Scheme::Archive }

	#[inline]
	pub fn to_archive(&self) -> Self {
		Self { loc: self.loc.clone(), scheme: Scheme::Archive, ..Default::default() }
	}

	#[inline]
	pub fn into_archive(mut self) -> Self {
		self.scheme = Scheme::Archive;
		self.frag = OsString::new();
		self
	}

	// --- Loc
	#[inline]
	pub fn set_loc(&mut self, loc: Loc) { self.loc = loc; }

	#[inline]
	pub fn to_path(&self) -> PathBuf { self.loc.to_path_buf() }

	#[inline]
	pub fn into_path(self) -> PathBuf { self.loc.into_path() }

	// --- Scheme
	#[inline]
	pub fn scheme(&self) -> Scheme { self.scheme }

	// --- Frag
	#[inline]
	pub fn frag(&self) -> &OsStr { &self.frag }
}

impl Hash for Url {
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.loc.hash(state);

		match self.scheme {
			Scheme::Regular | Scheme::SearchItem => {}
			Scheme::Search | Scheme::Archive => {
				self.scheme.hash(state);
				self.frag.hash(state);
			}
		}
	}
}

impl PartialEq for Url {
	fn eq(&self, other: &Self) -> bool {
		match (self.scheme, other.scheme) {
			(Scheme::Regular | Scheme::SearchItem, Scheme::Regular | Scheme::SearchItem) => {
				self.loc == other.loc
			}
			_ => self.loc == other.loc && self.scheme == other.scheme,
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
