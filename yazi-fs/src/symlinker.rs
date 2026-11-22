use std::{ffi::OsStr, fs, io::{self, ErrorKind}, path::Path};

use crate::ok_or_not_found;

pub struct Symlinker;

impl Symlinker {
	#[cfg(unix)]
	pub fn dir(original: &OsStr, link: &Path, force: bool) -> io::Result<()> {
		Self::file(original, link, force)
	}

	#[cfg(windows)]
	pub fn dir(original: &OsStr, link: &Path, force: bool) -> io::Result<()> {
		if force {
			Self::remove_existing(link)?;
		}

		std::os::windows::fs::symlink_dir(original, link)
	}

	pub fn file(original: &OsStr, link: &Path, force: bool) -> io::Result<()> {
		if force {
			Self::remove_existing(link)?;
		}

		#[cfg(unix)]
		{
			std::os::unix::fs::symlink(original, link)
		}
		#[cfg(windows)]
		{
			std::os::windows::fs::symlink_file(original, link)
		}
	}

	fn remove_existing(path: &Path) -> io::Result<()> {
		let error = match fs::remove_file(path) {
			Err(e) if matches!(e.kind(), ErrorKind::IsADirectory | ErrorKind::PermissionDenied) => e,
			result => return ok_or_not_found(result),
		};

		match fs::remove_dir(path) {
			Err(e) if e.kind() == ErrorKind::NotADirectory => Err(error),
			result => ok_or_not_found(result),
		}
	}
}
