use std::{fs::{Metadata, canonicalize, metadata, symlink_metadata}, io, os::unix::fs::MetadataExt, path::Path};

use super::Identity;

impl Identity {
	pub(super) fn same_entry_impl(a: &Path, b: &Path) -> io::Result<bool> {
		let (left, right) = (symlink_metadata(a)?, symlink_metadata(b)?);

		if left.ino() != right.ino() || left.dev() != right.dev() {
			Ok(false)
		} else if left.is_symlink() {
			Self::same_entry_path(a, b)
		} else {
			Ok(canonicalize(a)? == canonicalize(b)?)
		}
	}

	pub(super) fn same_file_impl(a: &Path, b: &Path) -> io::Result<bool> {
		let (left, right) = (metadata(a)?, metadata(b)?);

		Ok(left.ino() == right.ino() && left.dev() == right.dev())
	}

	pub(crate) fn same_symlink_target(a: &Path, b: &Path, right: &Metadata) -> io::Result<bool> {
		let left = metadata(a)?;

		Ok(
			left.ino() == right.ino()
				&& left.dev() == right.dev()
				&& (left.nlink() == 1 || Self::same_entry_path(a, b)?),
		)
	}

	pub(crate) fn same_hardlink_target(
		a: &Path,
		b: &Path,
		left: &Metadata,
		right: &Metadata,
	) -> io::Result<bool> {
		Ok(
			left.ino() == right.ino()
				&& left.dev() == right.dev()
				&& (left.nlink() == 1 || Self::same_entry_path(a, b)?),
		)
	}
}
