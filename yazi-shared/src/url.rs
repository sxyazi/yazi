use std::{ffi::{OsStr, OsString}, fmt::{Debug, Formatter}, ops::{Deref, DerefMut}, path::{Path, PathBuf, MAIN_SEPARATOR}};

#[derive(Clone, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Url {
	scheme: UrlScheme,
	path:   PathBuf,
	frag:   Option<String>,
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

	#[inline]
	pub fn was_hidden(&self) -> bool {
		self.file_name().map_or(false, |s| s.to_string_lossy().starts_with('.'))
	}

	#[inline]
	pub fn was_dir(&self) -> bool {
		// TODO: uncomment this when Rust 1.74 is released
		// let b = self.path.as_os_str().as_encoded_bytes();
		// if let [.., last] = b { *last == MAIN_SEPARATOR as u8 } else { false }

		#[cfg(unix)]
		{
			use std::os::unix::ffi::OsStrExt;
			let b = self.path.as_os_str().as_bytes();
			if let [.., last] = b { *last == MAIN_SEPARATOR as u8 } else { false }
		}

		#[cfg(windows)]
		{
			let s = self.path.to_string_lossy();
			let b = s.as_bytes();
			if let [.., last] = b { *last == MAIN_SEPARATOR as u8 } else { false }
		}
	}

	#[inline]
	pub fn into_dir(mut self) -> Self {
		if self.was_dir() {
			self
		} else {
			self.path.as_mut_os_string().push("/");
			self
		}
	}
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
	pub fn to_search(&self, frag: String) -> Self { self.clone().into_search(frag) }

	#[inline]
	pub fn into_search(mut self, frag: String) -> Self {
		self.scheme = UrlScheme::Search;
		self.frag = Some(frag);
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

	// --- Frag
	#[inline]
	pub fn frag(&self) -> Option<&str> { self.frag.as_deref() }
}
