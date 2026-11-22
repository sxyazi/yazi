use std::{fs, io::{self, ErrorKind}, path::Path};

#[cfg(unix)]
use rustix::fs::{AtFlags, CWD, linkat};
#[cfg(windows)]
use windows_sys::Win32::Storage::FileSystem::FILE_FLAG_OPEN_REPARSE_POINT;
use yazi_macro::ok_or_not_found;

use super::Local;
use crate::{identity::Identity, symlinker::Symlinker};

impl Local<'_> {
	#[cfg(unix)]
	pub(super) fn hardlink(from: &Path, to: &Path, follow: bool, force: bool) -> io::Result<()> {
		let flags = if follow { AtFlags::SYMLINK_FOLLOW } else { AtFlags::empty() };
		let create = |p: &Path| linkat(CWD, from, CWD, p, flags).map_err(io::Error::from);

		match create(to) {
			Err(e) if force && e.kind() == ErrorKind::AlreadyExists => {}
			result => return result,
		}

		let dest = ok_or_not_found!(fs::symlink_metadata(to), return create(to));
		if dest.is_dir() {
			return Err(io::Error::new(ErrorKind::IsADirectory, "Cannot overwrite a directory"));
		}

		let src = if follow { fs::metadata(from)? } else { fs::symlink_metadata(from)? };
		if Identity::same_hardlink_target(from, to, &src, &dest)? {
			return Err(io::Error::new(ErrorKind::InvalidInput, "Cannot link a file onto itself"));
		}

		Self::replace_hardlink(to, create)
	}

	#[cfg(windows)]
	pub(super) fn hardlink(from: &Path, to: &Path, follow: bool, force: bool) -> io::Result<()> {
		let resolved = follow.then(|| fs::canonicalize(from)).transpose()?;
		let create = |p: &Path| fs::hard_link(resolved.as_deref().unwrap_or(from), p);

		match create(to) {
			Err(e) if force && e.kind() == ErrorKind::AlreadyExists => {}
			result => return result,
		}

		let dest =
			ok_or_not_found!(Identity::open(to, FILE_FLAG_OPEN_REPARSE_POINT), return create(to));
		if dest.metadata()?.is_dir() {
			return Err(io::Error::new(ErrorKind::IsADirectory, "Cannot overwrite a directory"));
		}

		let src = Identity::open(from, if follow { 0 } else { FILE_FLAG_OPEN_REPARSE_POINT })?;
		if Identity::same_hardlink_target(from, to, &src, &dest)? {
			return Err(io::Error::new(ErrorKind::InvalidInput, "Cannot link a file onto itself"));
		}

		drop((src, dest));
		Self::replace_hardlink(to, create)
	}

	fn replace_hardlink(to: &Path, create: impl Fn(&Path) -> io::Result<()>) -> io::Result<()> {
		let tmp = Symlinker::temporary_link(to, create)?;
		let result = fs::rename(&tmp, to);
		_ = fs::remove_file(tmp); // Renaming two hardlinks to the same file can leave both names in place.
		result
	}
}
