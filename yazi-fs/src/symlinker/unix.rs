use std::{ffi::OsStr, fs, io::{self, ErrorKind}, os::unix::fs::symlink, path::Path};

use yazi_macro::ok_or_not_found;

use super::Symlinker;
use crate::identity::Identity;

impl Symlinker {
	pub fn dir(original: &OsStr, link: &Path, force: bool) -> io::Result<()> {
		Self::file(original, link, force)
	}

	pub fn file(original: &OsStr, link: &Path, force: bool) -> io::Result<()> {
		let original = Path::new(original);
		match symlink(original, link) {
			Err(e) if force && e.kind() == ErrorKind::AlreadyExists => {}
			result => return result,
		}

		let dest = ok_or_not_found!(fs::symlink_metadata(link), return symlink(original, link));
		if dest.is_dir() {
			return Err(io::Error::new(ErrorKind::IsADirectory, "Cannot overwrite a directory"));
		}

		let target = Self::resolve_target(original, link);
		if ok_or_not_found!(Identity::same_symlink_target(&target, link, &dest)) {
			return Err(io::Error::new(ErrorKind::InvalidInput, "Cannot link a file onto itself"));
		}

		let tmp = Self::temporary_link(link, |p| symlink(original, p))?;
		fs::rename(&tmp, link).inspect_err(|_| _ = fs::remove_file(&tmp))
	}
}
