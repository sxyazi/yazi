use std::{ffi::{OsStr, OsString}, fmt::{Debug, Display, Formatter}, ops::{Deref, DerefMut}, path::{Path, PathBuf}};

use percent_encoding::{percent_decode_str, percent_encode, AsciiSet, CONTROLS};
use serde::{Deserialize, Serialize};

const ENCODE_SET: &AsciiSet = &CONTROLS.add(b'#');

#[derive(Clone, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Url {
	scheme: UrlScheme,
	path:   PathBuf,
	frag:   String,
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
				url.frag = b.to_string();
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

impl Display for Url {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		if self.scheme == UrlScheme::Regular {
			return f.write_str(&self.path.to_string_lossy());
		}

		let scheme = match self.scheme {
			UrlScheme::Regular => unreachable!(),
			UrlScheme::Search => "search://",
			UrlScheme::Archive => "archive://",
		};
		let path = percent_encode(self.path.as_os_str().as_encoded_bytes(), ENCODE_SET);

		write!(f, "{scheme}{path}")?;
		if !self.frag.is_empty() {
			write!(f, "#{}", self.frag)?;
		}

		Ok(())
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

	#[cfg(unix)]
	#[inline]
	pub fn is_hidden(&self) -> bool {
		self.file_name().map_or(false, |s| s.as_encoded_bytes().starts_with(b"."))
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
		self.frag = frag;
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
