use std::{ffi::{OsStr, OsString}, fmt::{Debug, Formatter}, ops::{Deref, DerefMut}, path::{Path, PathBuf}};

use percent_encoding::{percent_decode_str, percent_encode, AsciiSet, CONTROLS};

const ENCODE_SET: &AsciiSet = &CONTROLS.add(b'#');

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
	fn from(path: String) -> Self { Self::from(path.as_str()) }
}

impl From<&String> for Url {
	fn from(path: &String) -> Self { Self::from(path.as_str()) }
}

impl From<&str> for Url {
	fn from(mut path: &str) -> Self {
		let mut url = Url::default();
		match path.split_once("://").map(|(a, b)| (UrlScheme::from(a), b)) {
			None => {
				url.path = PathBuf::from(path);
				return url;
			}
			Some((UrlScheme::Regular, b)) => {
				url.path = PathBuf::from(b);
				return url;
			}
			Some((a, b)) => {
				url.scheme = a;
				path = b;
			}
		}
		match path.split_once('#') {
			None => {
				url.path = percent_decode_str(path).decode_utf8_lossy().into_owned().into();
			}
			Some((a, b)) => {
				url.path = percent_decode_str(a).decode_utf8_lossy().into_owned().into();
				url.frag = Some(b.to_string()).filter(|s| !s.is_empty());
			}
		}
		url
	}
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

impl ToString for Url {
	fn to_string(&self) -> String {
		if self.scheme == UrlScheme::Regular {
			return self.path.to_string_lossy().to_string();
		}

		let scheme = match self.scheme {
			UrlScheme::Regular => unreachable!(),
			UrlScheme::Search => "search://",
			UrlScheme::Archive => "archive://",
		};

		let path = percent_encode(self.path.as_os_str().as_encoded_bytes(), ENCODE_SET);
		let frag = self.frag.as_ref().map(|s| format!("#{s}")).unwrap_or_default();
		format!("{scheme}{path}{frag}")
	}
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
	pub fn is_hidden(&self) -> bool {
		self.file_name().map_or(false, |s| s.as_encoded_bytes().starts_with(&[b'.']))
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

impl From<&str> for UrlScheme {
	fn from(value: &str) -> Self {
		match value {
			"search" => UrlScheme::Search,
			"archive" => UrlScheme::Archive,
			_ => UrlScheme::Regular,
		}
	}
}
