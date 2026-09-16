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
	match clear_no_reparse(path) {
		Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(()),
		result => result,
	}
}

#[cfg(windows)]
fn clear_no_reparse(path: &Path) -> io::Result<()> {
	use std::{fs::OpenOptions, os::windows::fs::{MetadataExt, OpenOptionsExt}};

	use windows_sys::Win32::Storage::FileSystem::{DELETE, FILE_ATTRIBUTE_DIRECTORY, FILE_ATTRIBUTE_REPARSE_POINT, FILE_FLAG_BACKUP_SEMANTICS, FILE_FLAG_OPEN_REPARSE_POINT, FILE_READ_ATTRIBUTES, FILE_SHARE_READ, FILE_SHARE_WRITE};

	let dir = OpenOptions::new()
		.access_mode(DELETE | FILE_READ_ATTRIBUTES)
		.share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE)
		.custom_flags(FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT)
		.open(path)?;

	let attrs = dir.metadata()?.file_attributes();
	if attrs & (FILE_ATTRIBUTE_DIRECTORY | FILE_ATTRIBUTE_REPARSE_POINT) != FILE_ATTRIBUTE_DIRECTORY {
		return Ok(());
	}

	if let Ok(entries) = std::fs::read_dir(path) {
		for dent in entries.flatten() {
			if dent.file_type().is_ok_and(|t| t.is_dir()) {
				clear_no_reparse(&dent.path()).ok();
			}
		}
	}

	remove_dir(&dir)
}

#[cfg(windows)]
fn remove_dir(dir: &std::fs::File) -> io::Result<()> {
	use std::{mem, os::windows::io::AsRawHandle};

	use windows_sys::Win32::{Foundation::HANDLE, Storage::FileSystem::{FILE_DISPOSITION_INFO, FileDispositionInfo, SetFileInformationByHandle}};
	use yazi_shim::bool_ok;

	let info = FILE_DISPOSITION_INFO { DeleteFile: true };
	bool_ok(unsafe {
		SetFileInformationByHandle(
			dir.as_raw_handle() as HANDLE,
			FileDispositionInfo,
			&info as *const FILE_DISPOSITION_INFO as _,
			mem::size_of::<FILE_DISPOSITION_INFO>() as u32,
		)
	})
}
