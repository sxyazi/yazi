use std::{ffi::OsStr, io, path::{Component, Path, PathBuf}};

pub trait PathExt {
	/// Canonicalize existing ancestors and append missing trailing components.
	fn canonicalize_partial(&self) -> io::Result<PathBuf>;

	fn has_parent_component(&self) -> bool;

	fn is_joinable(&self) -> bool;

	fn non_empty(&self) -> Option<&Path>;

	fn pair(&self) -> Option<(&Path, &OsStr)>;
}

impl PathExt for Path {
	fn canonicalize_partial(&self) -> io::Result<PathBuf> {
		match self.canonicalize() {
			Err(e) if e.kind() == io::ErrorKind::NotFound => {
				let (parent, name) = self.pair().ok_or(e)?;
				Ok(parent.canonicalize_partial()?.join(name))
			}
			result => result,
		}
	}

	fn has_parent_component(&self) -> bool {
		self.components().any(|c| matches!(c, Component::ParentDir))
	}

	fn is_joinable(&self) -> bool {
		!matches!(self.components().next(), Some(Component::Prefix(_) | Component::RootDir))
	}

	fn non_empty(&self) -> Option<&Path> { (!self.as_os_str().is_empty()).then_some(self) }

	fn pair(&self) -> Option<(&Path, &OsStr)> { self.parent().zip(self.file_name()) }
}
