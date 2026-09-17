use std::{ffi::{OsStr, OsString}, io::{self, ErrorKind}, os::{fd::AsFd, unix::ffi::OsStrExt}, path::Path};

use rustix::fs::{AtFlags, FileType, Mode, OFlags, Stat, open, statat};

use super::Casefold;
use crate::scanner::Scanner;

impl Casefold {
	pub(super) fn final_name(path: &Path) -> io::Result<OsString> {
		let (parent, name) = path
			.parent()
			.zip(path.file_name())
			.ok_or_else(|| io::Error::new(ErrorKind::InvalidInput, "Missing filename"))?;

		let parent = if parent.as_os_str().is_empty() { Path::new(".") } else { parent };
		let fd = open(parent, OFlags::RDONLY | OFlags::DIRECTORY | OFlags::CLOEXEC, Mode::empty())?;
		let meta = statat(&fd, name, AtFlags::SYMLINK_NOFOLLOW)?;

		if Self::case_sensitive(&fd) == Some(true) {
			Ok(name.to_owned())
		} else if Self::is_same_mount(&fd, name) {
			Self::scan_fast(name, &fd, meta)
		} else {
			Self::scan_full(name, &fd, meta)
		}
	}

	fn scan_fast(name: &OsStr, fd: &impl AsFd, stat: Stat) -> io::Result<OsString> {
		let mut names = vec![];
		Scanner::visit(&fd, |n, ino| {
			let n = OsStr::from_bytes(n.to_bytes());
			if n == name {
				names = vec![n.to_owned()];
				Ok(false)
			} else if ino == stat.st_ino {
				names.push(n.to_owned());
				Ok(stat.st_nlink != 1 || !FileType::from_raw_mode(stat.st_mode).is_file())
			} else {
				Ok(true)
			}
		})?;

		if names.is_empty() {
			Err(ErrorKind::NotFound.into())
		} else if names.len() == 1 {
			Ok(names.pop().unwrap())
		} else {
			Err(io::Error::other("Cannot determine the filename among multiple hardlinks"))
		}
	}

	fn scan_full(name: &OsStr, fd: &impl AsFd, stat: Stat) -> io::Result<OsString> {
		let (mut exact, mut names) = (false, vec![]);
		Scanner::visit(&fd, |n, _| {
			let n = OsStr::from_bytes(n.to_bytes());
			if n == name {
				exact = true;
				Ok(false)
			} else {
				names.push(n.to_owned());
				Ok(true)
			}
		})?;
		if exact {
			return Ok(name.to_owned());
		}

		let mut found = None;
		for n in names {
			match statat(&fd, &n, AtFlags::SYMLINK_NOFOLLOW) {
				Ok(s) if s.st_ino != stat.st_ino || s.st_dev != stat.st_dev => {}
				Ok(_) if found.replace(n).is_some() => {
					return Err(io::Error::other("Cannot determine the filename among multiple hardlinks"));
				}
				Ok(_) if stat.st_nlink == 1 && FileType::from_raw_mode(stat.st_mode).is_file() => break,
				Ok(_) | Err(rustix::io::Errno::NOENT) => {}
				Err(e) => return Err(e.into()),
			}
		}

		found.ok_or_else(|| ErrorKind::NotFound.into())
	}
}
