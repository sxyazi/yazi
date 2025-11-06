use hashbrown::Equivalent;
use serde::Serialize;

use super::{AsInnerView, AsPathView};
use crate::path::{PathBufLike, PathLike};

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum PathDyn<'p> {
	Os(&'p std::path::Path),
}

impl<'a> AsPathView<'a, PathDyn<'a>> for PathDyn<'a> {
	fn as_path_view(self) -> PathDyn<'a> { self }
}

impl<'a> AsPathView<'a, PathDyn<'a>> for std::path::Components<'a> {
	fn as_path_view(self) -> PathDyn<'a> { PathDyn::Os(self.as_path()) }
}

impl<'a> From<&'a std::path::Path> for PathDyn<'a> {
	fn from(value: &'a std::path::Path) -> Self { PathDyn::Os(value) }
}

impl<'a> From<&'a PathBufDyn> for PathDyn<'a> {
	fn from(value: &'a PathBufDyn) -> Self { value.borrow() }
}

impl<'p> PathLike<'p> for PathDyn<'p> {
	type Components<'a> = std::path::Components<'a>;
	type Display<'a> = std::path::Display<'a>;
	type Inner = &'p [u8];
	type Owned = PathBufDyn;
	type View<'a> = PathDyn<'a>;

	fn components(self) -> Self::Components<'p> {
		match self {
			Self::Os(p) => p.components(),
		}
	}

	// FIXME: remove
	fn default() -> Self { Self::Os(std::path::Path::new("")) }

	fn display(self) -> Self::Display<'p> {
		match self {
			Self::Os(p) => p.display(),
		}
	}

	fn encoded_bytes(self) -> &'p [u8] {
		match self {
			Self::Os(p) => p.as_os_str().as_encoded_bytes(),
		}
	}

	fn extension(self) -> Option<Self::Inner> {
		Some(match self {
			Self::Os(p) => p.extension()?.as_encoded_bytes(),
		})
	}

	fn file_name(self) -> Option<Self::Inner> {
		Some(match self {
			Self::Os(p) => p.file_name()?.as_encoded_bytes(),
		})
	}

	fn file_stem(self) -> Option<Self::Inner> {
		Some(match self {
			Self::Os(p) => p.file_stem()?.as_encoded_bytes(),
		})
	}

	// FIXME: remove
	unsafe fn from_encoded_bytes(bytes: &'p [u8]) -> Self {
		Self::Os(std::path::Path::new(unsafe { std::ffi::OsStr::from_encoded_bytes_unchecked(bytes) }))
	}

	fn join<'a, T>(self, base: T) -> Self::Owned
	where
		T: AsPathView<'a, Self::View<'a>>,
	{
		match (self, base.as_path_view()) {
			(Self::Os(p), PathDyn::Os(q)) => Self::Owned::Os(p.join(q)),
		}
	}

	fn owned(self) -> Self::Owned {
		match self {
			Self::Os(p) => Self::Owned::Os(p.to_path_buf()),
		}
	}

	fn parent(self) -> Option<Self> {
		Some(match self {
			Self::Os(p) => Self::Os(p.parent()?),
		})
	}

	fn strip_prefix<'a, T>(self, base: T) -> Option<Self>
	where
		T: AsPathView<'a, Self::View<'a>>,
	{
		Some(match (self, base.as_path_view()) {
			(Self::Os(p), PathDyn::Os(q)) => Self::Os(p.strip_prefix(q).ok()?),
		})
	}
}

impl PartialEq<PathBufDyn> for PathDyn<'_> {
	fn eq(&self, other: &PathBufDyn) -> bool { *self == other.borrow() }
}

impl PartialEq<PathDyn<'_>> for &std::path::Path {
	fn eq(&self, other: &PathDyn<'_>) -> bool { matches!(*other, PathDyn::Os(p) if p == *self) }
}

impl Equivalent<PathBufDyn> for PathDyn<'_> {
	fn equivalent(&self, key: &PathBufDyn) -> bool { *self == key.borrow() }
}

// --- PathBufDyn
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(untagged)]
pub enum PathBufDyn {
	Os(std::path::PathBuf),
}

impl PathBufDyn {
	pub const fn os_default() -> Self { Self::Os(std::path::PathBuf::new()) }
}

impl PathBufLike for PathBufDyn {
	type Borrowed<'a> = PathDyn<'a>;
	type Inner = Vec<u8>;
	type InnerRef<'a> = &'a [u8];

	fn borrow(&self) -> Self::Borrowed<'_> {
		match self {
			Self::Os(p) => Self::Borrowed::Os(p.as_path()),
		}
	}

	fn encoded_bytes(&self) -> &[u8] {
		match self {
			Self::Os(p) => p.as_os_str().as_encoded_bytes(),
		}
	}

	// FIXME: remove
	unsafe fn from_encoded_bytes(bytes: Vec<u8>) -> Self {
		Self::Os(std::path::PathBuf::from(unsafe {
			std::ffi::OsString::from_encoded_bytes_unchecked(bytes)
		}))
	}

	fn into_encoded_bytes(self) -> Vec<u8> {
		match self {
			Self::Os(p) => p.into_os_string().into_encoded_bytes(),
		}
	}

	fn set_file_name<T>(&mut self, name: T)
	where
		T: for<'a> AsInnerView<'a, Self::InnerRef<'a>>,
	{
		// TODO: introduce a new `PathInnerDyn`
		todo!()
	}

	fn take(&mut self) -> Self {
		match self {
			Self::Os(p) => Self::Os(std::mem::take(p)),
		}
	}
}

impl From<PathDyn<'_>> for PathBufDyn {
	fn from(value: PathDyn<'_>) -> Self {
		match value {
			PathDyn::Os(p) => Self::Os(p.to_path_buf()),
		}
	}
}

impl From<&PathBufDyn> for PathBufDyn {
	fn from(value: &PathBufDyn) -> Self { value.clone() }
}

impl PartialEq<PathDyn<'_>> for PathBufDyn {
	fn eq(&self, other: &PathDyn<'_>) -> bool { self.borrow() == *other }
}

impl PartialEq<PathDyn<'_>> for &PathBufDyn {
	fn eq(&self, other: &PathDyn<'_>) -> bool { self.borrow() == *other }
}

impl Equivalent<PathDyn<'_>> for PathBufDyn {
	fn equivalent(&self, key: &PathDyn<'_>) -> bool { self.borrow() == *key }
}
