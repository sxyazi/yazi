use std::{ffi::OsStr, fmt::{Debug, Display, Formatter}, hash::{Hash, Hasher}, ops::Deref, path::{Path, PathBuf}};

use percent_encoding::{AsciiSet, CONTROLS, percent_decode_str, percent_encode};
use serde::{Deserialize, Serialize};

use super::UrnBuf;
use crate::url::Loc;

const ENCODE_SET: &AsciiSet = &CONTROLS.add(b'#');

#[derive(Clone, Default, Eq, Ord, PartialEq, PartialOrd)]
pub struct Url {
	loc:    Loc,
	scheme: UrlScheme,
	frag:   String,
}

#[derive(Clone, Copy, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum UrlScheme {
	#[default]
	Regular,
	Search,
	SearchItem,
	Archive,
}

impl Deref for Url {
	type Target = Loc;

	fn deref(&self) -> &Self::Target { &self.loc }
}

impl Debug for Url {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		match self.scheme {
			UrlScheme::Regular => write!(f, "Regular({:?})", self.loc),
			UrlScheme::Search => write!(f, "Search({:?}, {})", self.loc, self.frag),
			UrlScheme::SearchItem => write!(f, "SearchItem({:?})", self.loc),
			UrlScheme::Archive => write!(f, "Archive({:?})", self.loc),
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

impl From<&str> for Url {
	fn from(mut path: &str) -> Self {
		let mut url = Url::default();
		match path.split_once("://").map(|(a, b)| (UrlScheme::from(a), b)) {
			None => {
				url.loc = Loc::new(PathBuf::from(path));
				return url;
			}
			Some((UrlScheme::Regular, b)) => {
				url.loc = Loc::new(PathBuf::from(b));
				return url;
			}
			Some((a, b)) => {
				url.scheme = a;
				path = b;
			}
		}
		match path.split_once('#') {
			None => {
				// FIXME: use `Loc::from(base, path)` instead
				url.loc = Loc::new(percent_decode_str(path).decode_utf8_lossy().into_owned().into());
			}
			Some((a, b)) => {
				// FIXME: use `Loc::from(base, path)` instead
				url.loc = Loc::new(percent_decode_str(a).decode_utf8_lossy().into_owned().into());
				url.frag = b.to_string();
			}
		}
		url
	}
}

impl From<String> for Url {
	fn from(path: String) -> Self { path.as_str().into() }
}

impl From<&String> for Url {
	fn from(path: &String) -> Self { path.as_str().into() }
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
		if matches!(self.scheme, UrlScheme::Regular | UrlScheme::SearchItem) {
			return write!(f, "{}", self.loc.display());
		}

		let scheme = match self.scheme {
			UrlScheme::Regular | UrlScheme::SearchItem => unreachable!(),
			UrlScheme::Search => "search://",
			UrlScheme::Archive => "archive://",
		};
		let path = percent_encode(self.loc.as_os_str().as_encoded_bytes(), ENCODE_SET);

		write!(f, "{scheme}{path}")?;
		if !self.frag.is_empty() {
			write!(f, "#{}", self.frag)?;
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
			UrlScheme::Regular => Self::from(self.loc.join(path)),
			UrlScheme::Search => {
				let loc = Loc::from(&self.loc, self.loc.join(path));
				Self::from(loc).into_search_item()
			}
			UrlScheme::SearchItem => {
				let loc = Loc::from(self.loc.base(), self.loc.join(path));
				Self::from(loc).into_search_item()
			}
			UrlScheme::Archive => Self::from(self.loc.join(path)).into_archive(),
		}
	}

	#[inline]
	pub fn parent_url(&self) -> Option<Url> {
		let p = self.loc.parent()?;
		Some(match self.scheme {
			UrlScheme::Regular | UrlScheme::Search => Self::from(p),
			UrlScheme::SearchItem => {
				if p == self.loc.base() {
					Self::from(p).into_search("")
				} else {
					Self::from(p).into_search_item()
				}
			}
			UrlScheme::Archive => Self::from(p),
		})
	}

	#[inline]
	pub fn pair(&self) -> Option<(Self, UrnBuf)> { Some((self.parent_url()?, self.loc.urn_owned())) }

	#[inline]
	pub fn rebase(&self, parent: &Path) -> Self {
		debug_assert!(self.is_regular());
		self.loc.rebase(parent).into()
	}
}

impl Url {
	// --- Regular
	#[inline]
	pub fn is_regular(&self) -> bool { self.scheme == UrlScheme::Regular }

	#[inline]
	pub fn to_regular(&self) -> Self {
		Self { loc: self.loc.clone(), scheme: UrlScheme::Regular, frag: String::new() }
	}

	#[inline]
	pub fn into_regular(mut self) -> Self {
		self.scheme = UrlScheme::Regular;
		self.frag = String::new();
		self
	}

	// --- Search
	#[inline]
	pub fn is_search(&self) -> bool { self.scheme == UrlScheme::Search }

	#[inline]
	pub fn to_search(&self, frag: &str) -> Self {
		Self { loc: self.loc.clone(), scheme: UrlScheme::Search, frag: frag.to_owned() }
	}

	#[inline]
	pub fn into_search(mut self, frag: &str) -> Self {
		self.scheme = UrlScheme::Search;
		self.frag = frag.to_owned();
		self
	}

	#[inline]
	pub fn is_search_item(&self) -> bool { self.scheme == UrlScheme::SearchItem }

	#[inline]
	pub fn into_search_item(mut self) -> Self {
		self.scheme = UrlScheme::SearchItem;
		self.frag = String::new();
		self
	}

	#[inline]
	pub fn is_archive(&self) -> bool { self.scheme == UrlScheme::Archive }

	#[inline]
	pub fn to_archive(&self) -> Self {
		Self { loc: self.loc.clone(), scheme: UrlScheme::Archive, frag: String::new() }
	}

	#[inline]
	pub fn into_archive(mut self) -> Self {
		self.scheme = UrlScheme::Archive;
		self.frag = String::new();
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
	pub fn scheme(&self) -> UrlScheme { self.scheme }

	// --- Frag
	#[inline]
	pub fn frag(&self) -> &str { &self.frag }
}

impl From<&str> for UrlScheme {
	fn from(value: &str) -> Self {
		match value {
			"search" => UrlScheme::Search,
			"archive" => UrlScheme::Archive,
			_ => UrlScheme::Regular,
		}
	}
}

impl Hash for Url {
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.loc.hash(state);

		match self.scheme {
			UrlScheme::Regular | UrlScheme::SearchItem => {}
			UrlScheme::Search | UrlScheme::Archive => {
				self.scheme.hash(state);
				self.frag.hash(state);
			}
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
		Ok(Url::from(s))
	}
}
