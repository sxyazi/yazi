use super::{PathBufDyn, PathDyn};
use crate::path::PathCow;

// --- DynPath
pub trait DynPath {
	fn dyn_path(&self) -> PathDyn<'_>;
}

impl DynPath for std::path::Path {
	fn dyn_path(&self) -> PathDyn<'_> { PathDyn::Os(self) }
}

impl DynPath for std::path::PathBuf {
	fn dyn_path(&self) -> PathDyn<'_> { PathDyn::Os(self) }
}

impl DynPath for typed_path::UnixPath {
	fn dyn_path(&self) -> PathDyn<'_> { PathDyn::Unix(self) }
}

impl DynPath for typed_path::UnixPathBuf {
	fn dyn_path(&self) -> PathDyn<'_> { PathDyn::Unix(self) }
}

impl DynPath for PathDyn<'_> {
	fn dyn_path(&self) -> PathDyn<'_> { *self }
}

impl DynPath for PathBufDyn {
	fn dyn_path(&self) -> PathDyn<'_> {
		match self {
			Self::Os(p) => PathDyn::Os(p),
			Self::Unix(p) => PathDyn::Unix(p),
		}
	}
}

impl DynPath for PathCow<'_> {
	fn dyn_path(&self) -> PathDyn<'_> {
		match self {
			PathCow::Borrowed(p) => *p,
			PathCow::Owned(p) => p.dyn_path(),
		}
	}
}

impl DynPath for super::Components<'_> {
	fn dyn_path(&self) -> PathDyn<'_> { self.path() }
}

// --- DynPathRef
pub trait DynPathRef<'a> {
	fn dyn_path_ref(self) -> PathDyn<'a>;
}

impl<'a> DynPathRef<'a> for PathDyn<'a> {
	fn dyn_path_ref(self) -> Self { self }
}

impl<'a> DynPathRef<'a> for &'a std::path::Path {
	fn dyn_path_ref(self) -> PathDyn<'a> { PathDyn::Os(self) }
}

impl<'a> DynPathRef<'a> for &'a typed_path::UnixPath {
	fn dyn_path_ref(self) -> PathDyn<'a> { PathDyn::Unix(self) }
}
