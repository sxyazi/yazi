use std::{fs::{File, OpenOptions}, io, os::windows::fs::OpenOptionsExt, thread};

use windows_sys::{Wdk::Storage::FileSystem::FILE_SYNCHRONOUS_IO_NONALERT, Win32::{Foundation::{ERROR_BAD_NET_NAME, ERROR_BAD_NETPATH, ERROR_DELETE_PENDING, ERROR_DIR_NOT_EMPTY, ERROR_DIRECTORY, ERROR_FILE_NOT_FOUND, ERROR_PATH_NOT_FOUND, ERROR_SHARING_VIOLATION}, Storage::FileSystem::{DELETE, FILE_ATTRIBUTE_DIRECTORY, FILE_FLAG_BACKUP_SEMANTICS, FILE_FLAG_OPEN_REPARSE_POINT, FILE_LIST_DIRECTORY, SYNCHRONIZE}}};
use yazi_shim::fs::{FileExt, ReadDir};

use super::Remover;

impl Remover {
	pub(super) fn remove(&mut self) -> io::Result<()> {
		let root = OpenOptions::new()
			.access_mode(FILE_LIST_DIRECTORY)
			.custom_flags(FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT)
			.open(&self.path)?;

		if root.basic_info()?.FileAttributes & FILE_ATTRIBUTE_DIRECTORY == 0 {
			return Err(io::Error::from_raw_os_error(ERROR_DIRECTORY as _));
		}

		self.check()?;
		let mut stack = vec![ReadDir::new(root)?];
		while let Some(entries) = stack.last_mut() {
			self.check()?;

			let Some(entry) = entries.next()? else {
				self.delete_at(entries.file(), &[], ERROR_DIR_NOT_EMPTY)?;
				stack.pop();
				continue;
			};
			if *entry.name == [b'.' as u16] || *entry.name == [b'.' as u16, b'.' as u16] {
				continue;
			}

			if !entry.is_dir() {
				self.delete_at(entry.dir, entry.name, ERROR_SHARING_VIOLATION)?;
			} else if let Some(child) = self.open_dir(entry.dir, entry.name)? {
				self.check()?;
				stack.push(ReadDir::new(child)?);
			}
		}

		Ok(())
	}

	fn delete_at(&mut self, parent: &File, name: &[u16], code: u32) -> io::Result<()> {
		for n in 0..50 {
			let result = self
				.open_at(parent, name, DELETE, 0)
				.and_then(|file| file.map(|file| file.delete()).transpose());

			match result {
				Ok(Some(())) => {
					self.progress.advance(&self.tx);
					return Ok(());
				}
				Ok(None) => return Ok(()),
				Err(e) if n < 49 && e.raw_os_error() == Some(code as _) => thread::yield_now(),
				Err(e) => return Err(e),
			}
		}
		unreachable!()
	}

	fn open_dir(&self, parent: &File, name: &[u16]) -> io::Result<Option<File>> {
		self.open_at(parent, name, FILE_LIST_DIRECTORY | SYNCHRONIZE, FILE_SYNCHRONOUS_IO_NONALERT)
	}

	fn open_at(
		&self,
		parent: &File,
		name: &[u16],
		access: u32,
		options: u32,
	) -> io::Result<Option<File>> {
		self.check()?;

		match parent.open_at(name, access, options) {
			Ok(file) => Ok(Some(file)),
			Err(e) if let Some(code) = e.raw_os_error() => match code as u32 {
				// The entry is absent; there is nothing left to remove.
				ERROR_FILE_NOT_FOUND => Ok(None),
				// A path component is absent; the entry is no longer reachable.
				ERROR_PATH_NOT_FOUND => Ok(None),
				// The network path is unavailable; treat it as absent, as std does.
				ERROR_BAD_NETPATH => Ok(None),
				// The network share is unavailable; treat it as absent, as std does.
				ERROR_BAD_NET_NAME => Ok(None),
				// Deletion is already pending; no further deletion is needed.
				ERROR_DELETE_PENDING => Ok(None),
				_ => Err(e),
			},
			Err(e) => Err(e),
		}
	}
}
