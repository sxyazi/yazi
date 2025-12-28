use std::{io, path::{Path, PathBuf}};

pub async fn match_name_case(path: impl AsRef<Path>) -> bool {
	let path = path.as_ref();
	casefold(path).await.is_ok_and(|p| p.file_name() == path.file_name())
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
#[allow(irrefutable_let_patterns)]
fn casefold_impl(path: PathBuf) -> io::Result<PathBuf> {
	use std::{ffi::{CString, OsStr, OsString}, fs::File, os::{fd::{AsRawFd, FromRawFd}, unix::{ffi::{OsStrExt, OsStringExt}, fs::MetadataExt}}};

	use libc::{O_NOFOLLOW, O_PATH};

	let cstr = CString::new(path.into_os_string().into_vec())?;
	let path = Path::new(OsStr::from_bytes(cstr.as_bytes()));
	let Some((parent, name)) = path.parent().zip(path.file_name()) else {
		return Ok(PathBuf::from(OsString::from_vec(cstr.into_bytes())));
	};

	let file = match unsafe { libc::open(cstr.as_ptr(), O_PATH | O_NOFOLLOW) } {
		ret if ret < 0 => return Err(io::Error::last_os_error()),
		ret => unsafe { File::from_raw_fd(ret) },
	};

	// Fast path: if the `/proc/self/fd/N` matches
	if let Ok(p) = std::fs::read_link(format!("/proc/self/fd/{}", file.as_raw_fd()))
		&& let path = path.as_os_str()
		&& let Some(b) = p.as_os_str().as_bytes().get(..path.len())
		&& b.eq_ignore_ascii_case(path.as_bytes())
	{
		let mut b = p.into_os_string().into_vec();
		b.truncate(path.len());
		return Ok(PathBuf::from(OsString::from_vec(b)));
	}

	// Fast path: if the file isn't a symlink
	let meta = file.metadata()?;
	if !meta.is_symlink()
		&& let Some(n) = path.canonicalize()?.file_name()
	{
		return Ok(parent.join(n));
	}

	// Fallback: scan the directory for matching inodes
	let mut names = vec![];
	for entry in std::fs::read_dir(parent)? {
		let entry = entry?;
		let n = entry.file_name(); // TODO: use `file_name_ref()` when stabilized

		if n == name {
			return Ok(PathBuf::from(OsString::from_vec(cstr.into_bytes())));
		} else if let m = entry.metadata()?
			&& m.ino() == meta.ino()
			&& m.dev() == meta.dev()
		{
			names.push(n);
		}
	}

	if names.len() == 1 {
		// No hardlink that shares the same inode
		Ok(parent.join(&names[0]))
	} else if let mut it = names.iter().enumerate().filter(|&(_, n)| n.eq_ignore_ascii_case(name))
		&& let Some((i, _)) = it.next()
		&& it.next().is_none()
	{
		// Case-insensitive match
		Ok(parent.join(&names[i]))
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
