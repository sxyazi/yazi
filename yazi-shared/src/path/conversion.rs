use super::{PathBufDyn, PathDyn};
use crate::path::PathCow;

// --- AsPath
pub trait AsPath {
	fn as_path(&self) -> PathDyn<'_>;
}

impl AsPath for std::path::Path {
	fn as_path(&self) -> PathDyn<'_> { PathDyn::Os(self) }
}

impl AsPath for std::path::PathBuf {
	fn as_path(&self) -> PathDyn<'_> { PathDyn::Os(self) }
}

impl AsPath for typed_path::UnixPath {
	fn as_path(&self) -> PathDyn<'_> { PathDyn::Unix(self) }
}

impl AsPath for typed_path::UnixPathBuf {
	fn as_path(&self) -> PathDyn<'_> { PathDyn::Unix(self) }
}

impl AsPath for PathDyn<'_> {
	fn as_path(&self) -> PathDyn<'_> { *self }
}

impl AsPath for PathBufDyn {
	fn as_path(&self) -> PathDyn<'_> {
		match self {
			Self::Os(p) => PathDyn::Os(p),
			Self::Unix(p) => PathDyn::Unix(p),
		}
	}
}

impl AsPath for PathCow<'_> {
	fn as_path(&self) -> PathDyn<'_> {
		match self {
			PathCow::Borrowed(p) => *p,
			PathCow::Owned(p) => p.as_path(),
		}
	}
}

impl AsPath for super::Components<'_> {
	fn as_path(&self) -> PathDyn<'_> { self.path() }
}

// --- AsPathRef
pub trait AsPathRef<'a> {
	fn as_path_ref(self) -> PathDyn<'a>;
}

impl<'a> AsPathRef<'a> for PathDyn<'a> {
	fn as_path_ref(self) -> Self { self }
}
