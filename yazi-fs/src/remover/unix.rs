use std::{ffi::{CStr, CString}, io, os::unix::ffi::OsStrExt};

use rustix::{fd::BorrowedFd, fs::{self, AtFlags, Dir, FileType, Mode, OFlags}, io::Errno};

use super::Remover;
use crate::ok_or_not_found;

impl Remover {
	pub(super) fn remove(&mut self) -> io::Result<()> {
		let root = CString::new(self.path.as_os_str().as_bytes())?;
		let dir = match self.open_dir(fs::CWD, &root) {
			Ok(dir) => dir,
			Err(e) if e.raw_os_error() == Some(libc::ENOENT) => return Err(e),
			Err(e) => return self.unlink_symlink(&root, e),
		};

		let mut stack = vec![(dir, None)];
		while let Some((dir, _)) = stack.last_mut() {
			self.check()?;

			if let Some(entry) = dir.next() {
				let entry = match entry {
					Ok(entry) => entry,
					Err(e) if e == Errno::NOENT && stack.len() > 1 => {
						stack.pop();
						continue;
					}
					Err(e) => return Err(e.into()),
				};

				let name = entry.file_name();
				if matches!(name.to_bytes(), b"." | b"..") {
					continue;
				}

				let fd = dir.fd()?;
				if !matches!(entry.file_type(), FileType::Directory | FileType::Unknown) {
					ok_or_not_found(self.unlink(fd, name, AtFlags::empty()))?;
				} else if let Some(child) = self.open_or_unlink(fd, name)? {
					stack.push((child, Some(entry)));
				}
				continue;
			}

			let (_, entry) = stack.pop().unwrap();
			let name = entry.as_ref().map_or(&*root, |e| e.file_name());
			let fd = if let Some((dir, _)) = stack.last() { dir.fd()? } else { fs::CWD };
			ok_or_not_found(self.unlink(fd, name, AtFlags::REMOVEDIR))?;
		}

		Ok(())
	}

	fn open_dir(&self, parent: BorrowedFd<'_>, name: &CStr) -> io::Result<Dir> {
		let flags = OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC;
		loop {
			self.check()?;
			match fs::openat(parent, name, flags, Mode::empty()) {
				Err(Errno::INTR) => continue,
				result => return Ok(Dir::new(result?)?),
			}
		}
	}

	fn open_or_unlink(&mut self, parent: BorrowedFd<'_>, name: &CStr) -> io::Result<Option<Dir>> {
		match self.open_dir(parent, name) {
			Ok(dir) => Ok(Some(dir)),
			Err(e) if e.raw_os_error() == Some(libc::ENOENT) => Ok(None),
			Err(e) if matches!(e.raw_os_error(), Some(libc::ENOTDIR | libc::ELOOP)) => {
				ok_or_not_found(self.unlink(parent, name, AtFlags::empty()))?;
				Ok(None)
			}
			Err(e) => Err(e),
		}
	}

	fn unlink(&mut self, parent: BorrowedFd<'_>, name: &CStr, flags: AtFlags) -> io::Result<()> {
		self.check()?;

		fs::unlinkat(parent, name, flags)?;
		self.progress.advance(&self.tx);
		Ok(())
	}

	fn unlink_symlink(&mut self, name: &CStr, error: io::Error) -> io::Result<()> {
		self.check()?;

		if let Ok(stat) = fs::statat(fs::CWD, name, AtFlags::SYMLINK_NOFOLLOW)
			&& FileType::from_raw_mode(stat.st_mode) == FileType::Symlink
		{
			return self.unlink(fs::CWD, name, AtFlags::empty());
		}
		Err(error)
	}
}
