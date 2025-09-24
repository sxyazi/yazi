use std::{io, path::{Path, PathBuf}};

pub async fn must_case_match(path: impl AsRef<Path>) -> bool {
	let path = path.as_ref();
	casefold(path).await.is_ok_and(|p| p == path)
}

pub(super) async fn casefold(path: impl AsRef<Path>) -> io::Result<PathBuf> {
	let path = path.as_ref().to_owned();
	tokio::task::spawn_blocking(move || casefold_impl(path)).await?
}

#[cfg(any(
	target_os = "macos",
	target_os = "netbsd",
	target_os = "openbsd",
	target_os = "freebsd",
	target_os = "windows"
))]
fn casefold_impl(path: PathBuf) -> io::Result<PathBuf> {
	let mut it = path.components();
	let mut parts = vec![];
	loop {
		let p = it.as_path();
		let q = final_path(p)?;
		if p != q {
			parts.push(q);
		} else if parts.is_empty() {
			return Ok(q);
		} else {
			break;
		}
		if it.next_back().is_none() {
			break;
		}
	}

	let mut buf = it.as_path().to_path_buf();
	for p in parts.into_iter().rev() {
		if let Some(name) = p.file_name() {
			buf.push(name);
		} else {
			return Err(io::Error::other("Cannot get filename"));
		}
	}
	Ok(buf)
}

#[cfg(any(target_os = "linux", target_os = "android"))]
fn casefold_impl(path: PathBuf) -> io::Result<PathBuf> {
	use std::{ffi::{CString, OsStr, OsString}, fs::File, os::{fd::{AsRawFd, FromRawFd}, unix::{ffi::{OsStrExt, OsStringExt}, fs::MetadataExt}}};

	use libc::{O_NOFOLLOW, O_PATH};

	let cstr = CString::new(path.into_os_string().into_vec())?;
	let path = Path::new(OsStr::from_bytes(cstr.as_bytes()));
	let Some(parent) = path.parent() else {
		return Ok(PathBuf::from(OsString::from_vec(cstr.into_bytes())));
	};

	let file = match unsafe { libc::open(cstr.as_ptr(), O_PATH | O_NOFOLLOW) } {
		ret if ret < 0 => return Err(io::Error::last_os_error()),
		ret => unsafe { File::from_raw_fd(ret) },
	};

	// Fast path: if the `/proc/self/fd/N` matches
	let oss = path.as_os_str();
	if let Ok(p) = std::fs::read_link(format!("/proc/self/fd/{}", file.as_raw_fd()))
		&& let Some(b) = p.as_os_str().as_bytes().get(..oss.len())
		&& b.eq_ignore_ascii_case(oss.as_bytes())
	{
		let mut b = p.into_os_string().into_vec();
		b.truncate(oss.len());
		return Ok(PathBuf::from(OsString::from_vec(b)));
	}

	// Fallback: scan the directory for matching inodes
	let meta = file.metadata()?;
	let mut entries: Vec<_> = std::fs::read_dir(parent)?
		.filter_map(Result::ok)
		.filter_map(|e| e.metadata().ok().map(|m| (e, m)))
		.filter(|(_, m)| m.dev() == meta.dev() && m.ino() == meta.ino())
		.map(|(e, _)| e.path())
		.collect();

	if entries.len() == 1 {
		// No hardlink that shares the same inode
		Ok(entries.remove(0))
	} else if let Some(i) = entries.iter().position(|p| p == path) {
		// Exact match
		Ok(entries.swap_remove(i))
	} else if let Some(i) = entries.iter().position(|p| p.as_os_str().eq_ignore_ascii_case(oss)) {
		// Case-insensitive match
		Ok(entries.swap_remove(i))
	} else {
		Err(io::Error::from(io::ErrorKind::NotFound))
	}
}

#[cfg(any(
	target_os = "macos",
	target_os = "netbsd",
	target_os = "openbsd",
	target_os = "freebsd"
))]
fn final_path(path: &Path) -> io::Result<PathBuf> {
	use std::{ffi::{CStr, CString, OsString}, os::{fd::{AsRawFd, FromRawFd, OwnedFd}, unix::ffi::{OsStrExt, OsStringExt}}};

	use libc::{F_GETPATH, O_RDONLY, O_SYMLINK, PATH_MAX};

	let cstr = CString::new(path.as_os_str().as_bytes())?;
	let fd = match unsafe { libc::open(cstr.as_ptr(), O_RDONLY | O_SYMLINK) } {
		ret if ret < 0 => return Err(io::Error::last_os_error()),
		ret => unsafe { OwnedFd::from_raw_fd(ret) },
	};

	let mut buf = [0u8; PATH_MAX as usize];
	if unsafe { libc::fcntl(fd.as_raw_fd(), F_GETPATH, buf.as_mut_ptr()) } < 0 {
		return Err(io::Error::last_os_error());
	}

	let cstr = unsafe { CStr::from_ptr(buf.as_ptr() as *const i8) };
	Ok(OsString::from_vec(cstr.to_bytes().to_vec()).into())
}

#[cfg(target_os = "windows")]
fn final_path(path: &Path) -> io::Result<PathBuf> {
	use std::{ffi::OsString, fs::File, os::windows::{ffi::OsStringExt, fs::OpenOptionsExt, io::AsRawHandle}};

	use windows_sys::Win32::{Foundation::HANDLE, Storage::FileSystem::{FILE_FLAG_BACKUP_SEMANTICS, FILE_FLAG_OPEN_REPARSE_POINT, GetFinalPathNameByHandleW, VOLUME_NAME_DOS}};
	use yazi_shared::Either;

	let file = std::fs::OpenOptions::new()
		.access_mode(0)
		.custom_flags(FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT)
		.open(path)?;

	fn inner(file: &File, buf: &mut [u16]) -> io::Result<Either<PathBuf, u32>> {
		let len = unsafe {
			GetFinalPathNameByHandleW(
				file.as_raw_handle() as HANDLE,
				buf.as_mut_ptr(),
				buf.len() as u32,
				VOLUME_NAME_DOS,
			)
		};

		Ok(if len == 0 {
			Err(io::Error::last_os_error())?
		} else if len as usize > buf.len() {
			Either::Right(len)
		} else if buf.starts_with(&[92, 92, 63, 92]) {
			Either::Left(PathBuf::from(OsString::from_wide(&buf[4..len as usize])))
		} else {
			Either::Left(PathBuf::from(OsString::from_wide(&buf[0..len as usize])))
		})
	}

	match inner(&file, &mut [0u16; 512])? {
		Either::Left(path) => Ok(path),
		Either::Right(len) => inner(&file, &mut vec![0u16; len as usize])?
			.left_or_err(|| io::Error::new(io::ErrorKind::InvalidData, "path too long")),
	}
}
