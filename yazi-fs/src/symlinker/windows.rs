use std::{ffi::OsStr, fs, io::{self, ErrorKind}, os::windows::fs::{symlink_dir, symlink_file}, path::Path};

use windows_sys::Win32::Storage::FileSystem::FILE_FLAG_OPEN_REPARSE_POINT;
use yazi_macro::ok_or_not_found;

use super::Symlinker;
use crate::identity::Identity;

impl Symlinker {
	pub fn dir(original: &OsStr, link: &Path, force: bool) -> io::Result<()> {
		Self::create(original, link, force, true)
	}

	pub fn file(original: &OsStr, link: &Path, force: bool) -> io::Result<()> {
		Self::create(original, link, force, false)
	}

	fn create(original: &OsStr, link: &Path, force: bool, is_dir: bool) -> io::Result<()> {
		let original = Path::new(original);
		let symlink = |p: &Path| {
			if is_dir { symlink_dir(original, p) } else { symlink_file(original, p) }
		};

		match symlink(link) {
			Err(e) if force && e.kind() == ErrorKind::AlreadyExists => {}
			result => return result,
		}

		let dest =
			ok_or_not_found!(Identity::open(link, FILE_FLAG_OPEN_REPARSE_POINT), return symlink(link));
		if dest.metadata()?.is_dir() {
			return Err(io::Error::new(ErrorKind::IsADirectory, "Cannot overwrite a directory"));
		}

		let target = Self::resolve_target(original, link);
		if ok_or_not_found!(Identity::same_symlink_target(&target, link, &dest)) {
			return Err(io::Error::new(ErrorKind::InvalidInput, "Cannot link a file onto itself"));
		}
		drop(dest);

		let tmp = Self::temporary_link(link, symlink)?;
		fs::rename(&tmp, link).inspect_err(|_| {
			_ = if is_dir { fs::remove_dir(&tmp) } else { fs::remove_file(&tmp) };
		})
	}
}
