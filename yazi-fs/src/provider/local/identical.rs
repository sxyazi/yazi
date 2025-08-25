use std::{io, path::{Path, PathBuf}};

#[inline]
pub async fn identical<P, Q>(a: P, b: Q) -> io::Result<bool>
where
	P: AsRef<Path>,
	Q: AsRef<Path>,
{
	let (a, b) = (a.as_ref().to_owned(), b.as_ref().to_owned());
	tokio::task::spawn_blocking(move || identical_impl(a, b)).await?
}

#[cfg(unix)]
fn identical_impl(a: PathBuf, b: PathBuf) -> io::Result<bool> {
	use std::os::unix::fs::MetadataExt;

	let (a_, b_) = (std::fs::symlink_metadata(&a)?, std::fs::symlink_metadata(&b)?);
	Ok(
		a_.ino() == b_.ino()
			&& a_.dev() == b_.dev()
			&& std::fs::canonicalize(a)? == std::fs::canonicalize(b)?,
	)
}

#[cfg(windows)]
fn identical_impl(a: PathBuf, b: PathBuf) -> io::Result<bool> {
	use std::{fs::OpenOptions, mem, os::windows::{fs::OpenOptionsExt, io::AsRawHandle}};

	use windows_sys::Win32::{Foundation::HANDLE, Storage::FileSystem::{FILE_FLAG_BACKUP_SEMANTICS, FILE_FLAG_OPEN_REPARSE_POINT, FILE_ID_INFO, FileIdInfo, GetFileInformationByHandleEx}};

	fn file_id(path: PathBuf) -> io::Result<(u64, u128)> {
		let file = OpenOptions::new()
			.access_mode(0)
			.custom_flags(FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT)
			.open(path)?;

		let mut info: FILE_ID_INFO = unsafe { mem::zeroed() };
		let ret = unsafe {
			GetFileInformationByHandleEx(
				file.as_raw_handle() as HANDLE,
				FileIdInfo,
				&mut info as *mut FILE_ID_INFO as _,
				mem::size_of::<FILE_ID_INFO>() as u32,
			)
		};

		if ret == 0 {
			Err(io::Error::last_os_error())
		} else {
			Ok((info.VolumeSerialNumber, u128::from_le_bytes(info.FileId.Identifier)))
		}
	}

	Ok(file_id(a)? == file_id(b)?)
}
