use std::{io, path::{Path, PathBuf}};

pub async fn valid_name_case(path: impl AsRef<Path>) -> io::Result<bool> {
	let path = path.as_ref().to_owned();
	tokio::task::spawn_blocking(move || valid_name_case_impl(path)).await?
}

#[cfg(any(
	target_os = "macos",
	target_os = "netbsd",
	target_os = "openbsd",
	target_os = "freebsd"
))]
fn valid_name_case_impl(path: PathBuf) -> io::Result<bool> {
	use std::{ffi::{CStr, CString, OsStr}, os::{fd::{AsRawFd, FromRawFd, OwnedFd}, unix::ffi::OsStrExt}};

	use libc::{F_GETPATH, O_RDONLY, O_SYMLINK, PATH_MAX};

	let cstr = CString::new(path.into_os_string().into_encoded_bytes())?;
	let Some(name) = Path::new(OsStr::from_bytes(cstr.as_bytes())).file_name() else {
		return Ok(true);
	};

	let fd = match unsafe { libc::open(cstr.as_ptr(), O_RDONLY | O_SYMLINK) } {
		ret if ret < 0 => return Err(io::Error::last_os_error()),
		ret => unsafe { OwnedFd::from_raw_fd(ret) },
	};

	let mut buf = [0u8; PATH_MAX as usize];
	if unsafe { libc::fcntl(fd.as_raw_fd(), F_GETPATH, buf.as_mut_ptr()) } < 0 {
		return Err(io::Error::last_os_error());
	}

	Ok(
		unsafe { CStr::from_ptr(buf.as_ptr() as *const i8) }
			.to_bytes()
			.ends_with(name.as_encoded_bytes()),
	)
}

#[cfg(any(target_os = "linux", target_os = "android"))]
fn valid_name_case_impl(path: PathBuf) -> io::Result<bool> {
	use std::{ffi::{CString, OsStr}, fs::File, os::{fd::{AsRawFd, FromRawFd}, unix::ffi::OsStrExt}};

	use libc::{O_NOFOLLOW, O_PATH};

	let cstr = CString::new(path.into_os_string().into_encoded_bytes())?;
	let path = Path::new(OsStr::from_bytes(cstr.as_bytes()));
	let Some(name) = path.file_name() else { return Ok(true) };

	let file = match unsafe { libc::open(cstr.as_ptr(), O_PATH | O_NOFOLLOW) } {
		ret if ret < 0 => return Err(io::Error::last_os_error()),
		ret => unsafe { File::from_raw_fd(ret) },
	};

	Ok(if file.metadata()?.is_symlink() {
		std::fs::read_link(format!("/proc/self/fd/{}", file.as_raw_fd()))?.starts_with(path)
	} else {
		std::fs::canonicalize(path)?.file_name() == Some(name)
	})
}

#[cfg(target_os = "windows")]
fn valid_name_case_impl(path: PathBuf) -> io::Result<bool> {
	use std::{ffi::OsString, os::windows::{ffi::OsStringExt, fs::OpenOptionsExt, io::AsRawHandle}};

	use windows_sys::Win32::{Foundation::{HANDLE, MAX_PATH}, Storage::FileSystem::{FILE_FLAG_BACKUP_SEMANTICS, FILE_FLAG_OPEN_REPARSE_POINT, GetFinalPathNameByHandleW, VOLUME_NAME_DOS}};

	let Some(name) = path.file_name() else { return Ok(true) };

	let file = std::fs::OpenOptions::new()
		.access_mode(0)
		.custom_flags(FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT)
		.open(&path)?;

	let mut buf = [0u16; MAX_PATH as usize];
	let len = unsafe {
		GetFinalPathNameByHandleW(
			file.as_raw_handle() as HANDLE,
			buf.as_mut_ptr(),
			buf.len() as u32,
			VOLUME_NAME_DOS,
		)
	};

	if len == 0 {
		Err(io::Error::last_os_error())
	} else {
		Ok(PathBuf::from(OsString::from_wide(&buf[0..len as usize])).file_name() == Some(name))
	}
}
