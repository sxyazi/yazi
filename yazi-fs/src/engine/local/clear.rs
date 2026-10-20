use std::{io, path::Path};

#[cfg(unix)]
pub(super) fn remove_dir_clean_impl(path: &Path) -> io::Result<()> {
	use std::{fs::OpenOptions, io::ErrorKind, os::unix::fs::OpenOptionsExt};

	use rustix::{fs::OFlags, io::Errno};

	let dir = match OpenOptions::new()
		.read(true)
		.custom_flags((OFlags::DIRECTORY | OFlags::NOFOLLOW).bits() as _)
		.open(path)
	{
		Ok(dir) => dir,
		Err(e) if e.kind() == ErrorKind::NotFound => return Ok(()),
		Err(e) if e.kind() == ErrorKind::NotADirectory => return Ok(()),
		Err(e) if e.raw_os_error() == Some(Errno::LOOP.raw_os_error()) => return Ok(()),
		Err(e) => return Err(e),
	};

	clear_no_follow(dir.into());
	std::fs::remove_dir(path)
}

#[cfg(unix)]
fn clear_no_follow(dir: rustix::fd::OwnedFd) {
	use rustix::fs::{self, AtFlags, Dir, FileType, Mode, OFlags};

	let Ok(mut stream) = Dir::new(dir) else {
		return;
	};

	while let Some(Ok(entry)) = stream.next() {
		let name = entry.file_name();
		if matches!(name.to_bytes(), b"." | b"..") {
			continue;
		}

		if !matches!(entry.file_type(), FileType::Directory | FileType::Unknown) {
			continue;
		}

		let Ok(fd) = stream.fd() else { break };
		let Ok(child) = fs::openat(
			fd,
			name,
			OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
			Mode::empty(),
		) else {
			continue;
		};

		clear_no_follow(child);
		fs::unlinkat(fd, name, AtFlags::REMOVEDIR).ok();
	}
}

// --- Windows
#[cfg(windows)]
pub(super) fn remove_dir_clean_impl(path: &Path) -> io::Result<()> {
	use std::{fs::OpenOptions, os::windows::fs::OpenOptionsExt};

	use windows_sys::Win32::Storage::FileSystem::{DELETE, FILE_FLAG_BACKUP_SEMANTICS, FILE_FLAG_OPEN_REPARSE_POINT, FILE_READ_ATTRIBUTES, FILE_SHARE_READ, FILE_SHARE_WRITE};

	crate::ok_or_not_found(
		OpenOptions::new()
			.access_mode(DELETE | FILE_READ_ATTRIBUTES)
			.share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE)
			.custom_flags(FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT)
			.open(path)
			.and_then(clear_no_reparse),
	)
}

#[cfg(windows)]
fn clear_no_reparse(dir: std::fs::File) -> io::Result<()> {
	use windows_sys::{Wdk::Storage::FileSystem::{FILE_OPEN_FOR_BACKUP_INTENT, FILE_SYNCHRONOUS_IO_NONALERT}, Win32::Storage::FileSystem::{DELETE, FILE_ATTRIBUTE_DIRECTORY, FILE_ATTRIBUTE_REPARSE_POINT, FILE_LIST_DIRECTORY, FILE_READ_ATTRIBUTES, SYNCHRONIZE}};
	use yazi_shim::fs::{FileExt, ReadDir};

	let attrs = dir.basic_info()?.FileAttributes;
	if attrs & (FILE_ATTRIBUTE_DIRECTORY | FILE_ATTRIBUTE_REPARSE_POINT) != FILE_ATTRIBUTE_DIRECTORY {
		return Ok(());
	}

	let entries = dir.open_at(&[], FILE_LIST_DIRECTORY | SYNCHRONIZE, FILE_SYNCHRONOUS_IO_NONALERT);
	if let Ok(mut entries) = entries.and_then(ReadDir::new) {
		while let Ok(Some(entry)) = entries.next() {
			if *entry.name == [b'.' as u16] || *entry.name == [b'.' as u16, b'.' as u16] {
				continue;
			}

			if entry.is_dir()
				&& !entry.is_reparse()
				&& let Ok(child) =
					entry.dir.open_at(entry.name, DELETE | FILE_READ_ATTRIBUTES, FILE_OPEN_FOR_BACKUP_INTENT)
			{
				clear_no_reparse(child).ok();
			}
		}
	}

	dir.delete()
}
