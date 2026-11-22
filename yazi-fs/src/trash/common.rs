use std::{fs, io::{self, ErrorKind}, path::Path};

use super::{Trash, TrashEntry};

impl Trash {
	pub(crate) fn rename(entry: &TrashEntry, to: &Path) -> io::Result<()> {
		fs::rename(&entry.backing, to)?;
		if entry.has_rel() {
			return Ok(());
		}

		// Renaming onto the same file can succeed without removing the source name.
		match fs::symlink_metadata(&entry.backing) {
			Err(e) if e.kind() == ErrorKind::NotFound => Self::forget(entry),
			Err(e) => Err(e),
			Ok(_) => Ok(()),
		}
	}

	#[cfg(trash_unix)]
	pub(super) fn restore_item(from: &Path, to: &Path) -> io::Result<()> {
		let is_dir = fs::symlink_metadata(from)?.is_dir();
		if let Some(parent) = to.parent() {
			fs::create_dir_all(parent)?;
		}

		match if is_dir { fs::create_dir(to) } else { fs::File::create_new(to).map(drop) } {
			Ok(()) => fs::rename(from, to),
			Err(e) if e.kind() == ErrorKind::AlreadyExists => Err(io::Error::new(
				ErrorKind::AlreadyExists,
				format!("restore target already exists: {to:?}"),
			)),
			Err(e) => Err(e),
		}
	}

	pub(crate) fn set_original(entry: &TrashEntry, original: &Path) -> io::Result<()> {
		if entry.has_rel() {
			return Err(io::Error::new(ErrorKind::InvalidInput, "not a top-level trash item"));
		}
		if original.as_os_str().as_encoded_bytes().contains(&0) {
			return Err(io::Error::new(ErrorKind::InvalidInput, "invalid trash original path"));
		}
		if !original.is_absolute() || original.file_name().is_none() {
			return Err(io::Error::new(ErrorKind::InvalidInput, "invalid trash original path"));
		}

		entry.file_type()?;
		Self::write_info(entry, original, None)
	}
}
