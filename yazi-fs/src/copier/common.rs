use std::{fs::{File, Metadata}, io::{self, ErrorKind}, os::unix::fs::MetadataExt};

use super::Copier;

impl Copier {
	pub(super) fn file_meta(file: &File) -> io::Result<Metadata> {
		let meta = file.metadata()?;

		if !meta.is_file() {
			return Err(io::Error::new(ErrorKind::InvalidInput, "File is not a regular file"));
		}

		Ok(meta)
	}

	pub(super) fn ensure_distinct(src: &Metadata, dist: &File) -> io::Result<()> {
		let dist = Self::file_meta(dist)?;

		if src.ino() == dist.ino() && src.dev() == dist.dev() {
			Err(io::Error::new(ErrorKind::InvalidInput, "Cannot copy a file onto itself"))
		} else {
			Ok(())
		}
	}

	pub(super) fn skip_attrs(error: io::Error) -> io::Result<()> {
		match error.kind() {
			ErrorKind::PermissionDenied | ErrorKind::Unsupported => Ok(()),
			_ => Err(error),
		}
	}

	pub(super) fn apply_times(&self, file: &File) -> io::Result<()> {
		if let Ok(times) = self.attrs.try_into() {
			file.set_times(times).ok();
		}

		Ok(())
	}
}
