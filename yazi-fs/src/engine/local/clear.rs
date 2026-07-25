use std::{io, path::Path};

#[cfg(unix)]
pub(super) fn remove_dir_clean_impl(path: &Path) -> io::Result<()> {
	use std::{fs::OpenOptions, os::unix::fs::OpenOptionsExt};

	use libc::{ELOOP, ENOENT, ENOTDIR, O_DIRECTORY, O_NOFOLLOW};

	let dir = match OpenOptions::new().read(true).custom_flags(O_DIRECTORY | O_NOFOLLOW).open(path) {
		Ok(dir) => dir,
		Err(e) if e.raw_os_error().is_some_and(|e| matches!(e, ENOENT | ENOTDIR | ELOOP)) => {
			return Ok(());
		}
		Err(e) => return Err(e),
	};

	clear_no_follow(dir);
	std::fs::remove_dir(path)
}

#[cfg(unix)]
fn clear_no_follow(dir: std::fs::File) {
	use std::{ffi::CStr, os::fd::{AsRawFd, FromRawFd, IntoRawFd}};

	use libc::{AT_REMOVEDIR, DT_DIR, DT_UNKNOWN, O_CLOEXEC, O_DIRECTORY, O_NOFOLLOW, O_RDONLY};

	struct Dropper(*mut libc::DIR);

	impl Drop for Dropper {
		fn drop(&mut self) { _ = unsafe { libc::closedir(self.0) }; }
	}

	let stream = unsafe { libc::fdopendir(dir.as_raw_fd()) };
	if stream.is_null() {
		return;
	}

	let _ = dir.into_raw_fd();
	let stream = Dropper(stream);
	let fd = unsafe { libc::dirfd(stream.0) };

	loop {
		let entry = unsafe { libc::readdir(stream.0) };
		if entry.is_null() {
			break;
		}

		let name = unsafe { CStr::from_ptr((*entry).d_name.as_ptr()) };
		if matches!(name.to_bytes(), b"." | b"..") {
			continue;
		}

		let ty = unsafe { (*entry).d_type };
		if ty != DT_DIR && ty != DT_UNKNOWN {
			continue;
		}

		let child =
			unsafe { libc::openat(fd, name.as_ptr(), O_RDONLY | O_DIRECTORY | O_NOFOLLOW | O_CLOEXEC) };
		if child < 0 {
			continue;
		}

		clear_no_follow(unsafe { std::fs::File::from_raw_fd(child) });
		_ = unsafe { libc::unlinkat(fd, name.as_ptr(), AT_REMOVEDIR) };
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

	let info = FILE_DISPOSITION_INFO { DeleteFile: true };
	if unsafe {
		SetFileInformationByHandle(
			dir.as_raw_handle() as HANDLE,
			FileDispositionInfo,
			&info as *const FILE_DISPOSITION_INFO as _,
			mem::size_of::<FILE_DISPOSITION_INFO>() as u32,
		)
	} == 0
	{
		Err(io::Error::last_os_error())
	} else {
		Ok(())
	}
}
