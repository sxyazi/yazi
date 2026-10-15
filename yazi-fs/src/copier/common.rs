use std::{fs::{File, Metadata}, io};

use super::Copier;

impl Copier {
	pub(super) fn file_meta(file: &File) -> io::Result<Metadata> {
		let meta = file.metadata()?;

		if !meta.is_file() {
			return Err(io::Error::new(io::ErrorKind::InvalidInput, "File is not a regular file"));
		}

		Ok(meta)
	}

	pub(super) fn skip_attrs(error: io::Error) -> io::Result<()> {
		match error.kind() {
			io::ErrorKind::PermissionDenied | io::ErrorKind::Unsupported => Ok(()),
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
