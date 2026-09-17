use std::{ffi::OsString, fs::{File, OpenOptions}, io, mem, os::windows::{ffi::OsStringExt, fs::OpenOptionsExt, io::AsRawHandle}, path::{Component, Path}, ptr};

use either::Either;
use io::ErrorKind;
use windows_sys::Win32::{Foundation::{HANDLE, INVALID_HANDLE_VALUE}, Storage::FileSystem::{FILE_FLAG_BACKUP_SEMANTICS, FILE_FLAG_OPEN_REPARSE_POINT, FindClose, FindExInfoBasic, FindExSearchNameMatch, FindFirstFileExW, GetFinalPathNameByHandleW, VOLUME_NAME_DOS, WIN32_FIND_DATAW}};
use yazi_shim::ToWide;

use super::Casefold;

impl Casefold {
	pub(super) fn final_name(path: &Path) -> io::Result<OsString> {
		if let Ok(name) = Self::by_find(path) {
			return Ok(name);
		}

		let file = OpenOptions::new()
			.access_mode(0)
			.custom_flags(FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT)
			.open(path)?;

		match Self::by_handle(&file, &mut [0u16; 512])? {
			Either::Left(name) => Ok(name),
			Either::Right(len) => match Self::by_handle(&file, &mut vec![0u16; len as usize])? {
				Either::Left(name) => Ok(name),
				Either::Right(_) => Err(io::Error::new(ErrorKind::InvalidData, "path too long")),
			},
		}
	}

	fn by_find(path: &Path) -> io::Result<OsString> {
		Self::check_name(path)?;
		let wide = path.to_wide();

		let mut data = unsafe { mem::zeroed::<WIN32_FIND_DATAW>() };
		let handle = unsafe {
			FindFirstFileExW(
				wide.as_ptr(),
				FindExInfoBasic,
				(&raw mut data).cast(),
				FindExSearchNameMatch,
				ptr::null(),
				0,
			)
		};

		match handle {
			INVALID_HANDLE_VALUE => return Err(io::Error::last_os_error()),
			h => _ = unsafe { FindClose(h) },
		}

		Ok(OsString::from_wide(data.cFileName.split(|&c| c == 0).next().unwrap()))
	}

	fn by_handle(file: &File, buf: &mut [u16]) -> io::Result<Either<OsString, u32>> {
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
		} else if len as usize >= buf.len() {
			Either::Right(len)
		} else {
			Either::Left(OsString::from_wide(
				buf[..len as usize]
					.rsplit(|&c| c == b'\\' as u16)
					.next()
					.filter(|name| !name.is_empty())
					.ok_or_else(|| io::Error::other("Cannot get filename"))?,
			))
		})
	}

	fn check_name(path: &Path) -> io::Result<()> {
		if path.as_os_str().as_encoded_bytes().contains(&0) {
			return Err(ErrorKind::InvalidInput.into());
		}

		if path.components().any(|c| {
			matches!(
				c,
				Component::Normal(n)
					if n.as_encoded_bytes().iter().any(|&c| matches!(c, b'*' | b':' | b'?'))
			)
		}) {
			return Err(ErrorKind::InvalidInput.into());
		}

		Ok(())
	}
}
