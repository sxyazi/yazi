use std::{ffi::{OsStr, OsString}, ops::{Deref, DerefMut}, path::{Path, PathBuf}};

#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Url {
	scheme: UrlScheme,
	path:   PathBuf,
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum UrlScheme {
	#[default]
	None,
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
	pub fn new(url: impl Into<Url>, ctx: &Url) -> Self {
		let mut url: Self = url.into();
		url.scheme = ctx.scheme;
		url
	}

	#[inline]
	pub fn join(&self, path: impl AsRef<Path>) -> Self { Self::new(self.path.join(path), self) }

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
	pub fn is_none(&self) -> bool { self.scheme == UrlScheme::None }

	#[inline]
	pub fn is_search(&self) -> bool { self.scheme == UrlScheme::Search }

	#[inline]
	pub fn to_search(&self) -> Self {
		let mut url = self.clone();
		url.scheme = UrlScheme::Search;
		url
	}

	#[inline]
	pub fn is_archive(&self) -> bool { self.scheme == UrlScheme::Archive }

	// --- Path
	#[inline]
	pub fn set_path(&mut self, path: PathBuf) { self.path = path; }

	#[inline]
	pub fn parent_url(&self) -> Option<Url> { self.path.parent().map(|p| Self::new(p, self)) }
}
