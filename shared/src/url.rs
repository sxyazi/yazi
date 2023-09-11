use std::{ffi::{OsStr, OsString}, fmt::{Debug, Formatter}, ops::{Deref, DerefMut}, path::{Path, PathBuf}};

#[derive(Clone, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Url {
	scheme: UrlScheme,
	path:   PathBuf,
}

#[derive(Clone, Copy, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum UrlScheme {
	#[default]
	Regular,
	Search,
	Archive,
}

impl Deref for Url {
	type Target = PathBuf;

	fn deref(&self) -> &Self::Target { &self.path }
}

impl DerefMut for Url {
	fn deref_mut(&mut self) -> &mut Self::Target { &mut self.path }
}

impl Debug for Url {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result { write!(f, "{}", self.path.display()) }
}

impl From<PathBuf> for Url {
	fn from(path: PathBuf) -> Self { Self { path, ..Default::default() } }
}

impl From<&PathBuf> for Url {
	fn from(path: &PathBuf) -> Self { Self::from(path.clone()) }
}

impl From<&Path> for Url {
	fn from(path: &Path) -> Self { Self::from(path.to_path_buf()) }
}

impl From<String> for Url {
	fn from(path: String) -> Self { Self::from(PathBuf::from(path)) }
}

impl From<&String> for Url {
	fn from(path: &String) -> Self { Self::from(PathBuf::from(path)) }
}

impl From<&str> for Url {
	fn from(path: &str) -> Self { Self::from(PathBuf::from(path)) }
}

impl AsRef<Url> for Url {
	fn as_ref(&self) -> &Url { self }
}

impl AsRef<Path> for Url {
	fn as_ref(&self) -> &Path { &self.path }
}

impl AsRef<OsStr> for Url {
	fn as_ref(&self) -> &OsStr { self.path.as_os_str() }
}

impl Url {
	#[inline]
	pub fn join(&self, path: impl AsRef<Path>) -> Self {
		let url = Self::from(self.path.join(path));
		match self.scheme {
			UrlScheme::Regular => url,
			UrlScheme::Search => url,
			UrlScheme::Archive => url.into_archive(),
		}
	}

	#[inline]
	pub fn parent_url(&self) -> Option<Url> {
		self.path.parent().map(|p| {
			let url = Self::from(p);
			match self.scheme {
				UrlScheme::Regular => url,
				UrlScheme::Search => url,
				UrlScheme::Archive => url,
			}
		})
	}

	#[inline]
	pub fn strip_prefix(&self, base: impl AsRef<Path>) -> Option<&Path> {
		self.path.strip_prefix(base).ok()
	}

	#[inline]
	pub fn into_os_string(self) -> OsString { self.path.into_os_string() }
}

impl Url {
	// --- Scheme
	#[inline]
	pub fn is_regular(&self) -> bool { self.scheme == UrlScheme::Regular }

	#[inline]
	pub fn to_regular(&self) -> Self { self.clone().into_regular() }

	#[inline]
	pub fn into_regular(mut self) -> Self {
		self.scheme = UrlScheme::Regular;
		self
	}

	#[inline]
	pub fn is_search(&self) -> bool { self.scheme == UrlScheme::Search }

	#[inline]
	pub fn to_search(&self) -> Self { self.clone().into_search() }

	#[inline]
	pub fn into_search(mut self) -> Self {
		self.scheme = UrlScheme::Search;
		self
	}

	#[inline]
	pub fn is_archive(&self) -> bool { self.scheme == UrlScheme::Archive }

	#[inline]
	pub fn to_archive(&self) -> Self { self.clone().into_archive() }

	#[inline]
	pub fn into_archive(mut self) -> Self {
		self.scheme = UrlScheme::Archive;
		self
	}

	// --- Path
	#[inline]
	pub fn set_path(&mut self, path: PathBuf) { self.path = path; }
}
