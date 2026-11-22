use std::{fs::{File, OpenOptions}, io, os::windows::{fs::OpenOptionsExt, io::AsRawHandle}, path::Path};

use windows_sys::Win32::{Foundation::HANDLE, Storage::FileSystem::{FILE_FLAG_BACKUP_SEMANTICS, FILE_FLAG_OPEN_REPARSE_POINT, FILE_ID_INFO, FILE_STANDARD_INFO, FileIdInfo, FileStandardInfo, GetFileInformationByHandleEx}};
use yazi_shim::bool_ok;

use super::Identity;

impl Identity {
	pub(super) fn same_entry_impl(a: &Path, b: &Path) -> io::Result<bool> {
		let left = Self::open(a, FILE_FLAG_OPEN_REPARSE_POINT)?;
		let right = Self::open(b, FILE_FLAG_OPEN_REPARSE_POINT)?;

		if Self::file_id(&left)? == Self::file_id(&right)? {
			Self::same_entry_path(a, b)
		} else {
			Ok(false)
		}
	}

	pub(crate) fn same_file_impl(a: &Path, b: &Path) -> io::Result<bool> {
		let left = Self::open(a, 0)?;
		let right = Self::open(b, 0)?;

		Ok(Self::file_id(&left)? == Self::file_id(&right)?)
	}

	pub(crate) fn same_symlink_target(a: &Path, b: &Path, right: &File) -> io::Result<bool> {
		let left = Self::open(a, 0)?;
		if Self::file_id(&left)? != Self::file_id(right)? {
			return Ok(false);
		}

		let mut info = FILE_STANDARD_INFO::default();
		bool_ok(unsafe {
			GetFileInformationByHandleEx(
				left.as_raw_handle() as HANDLE,
				FileStandardInfo,
				(&raw mut info).cast(),
				size_of::<FILE_STANDARD_INFO>() as u32,
			)
		})?;

		Ok(info.NumberOfLinks == 1 || Self::same_entry_path(a, b)?)
	}

	pub(crate) fn same_hardlink_target(
		a: &Path,
		b: &Path,
		left: &File,
		right: &File,
	) -> io::Result<bool> {
		if Self::file_id(left)? != Self::file_id(right)? {
			return Ok(false);
		}

		let mut info = FILE_STANDARD_INFO::default();
		bool_ok(unsafe {
			GetFileInformationByHandleEx(
				left.as_raw_handle() as HANDLE,
				FileStandardInfo,
				(&raw mut info).cast(),
				size_of::<FILE_STANDARD_INFO>() as u32,
			)
		})?;

		Ok(info.NumberOfLinks == 1 || Self::same_entry_path(a, b)?)
	}

	pub(crate) fn open(path: &Path, flags: u32) -> io::Result<File> {
		OpenOptions::new().access_mode(0).custom_flags(FILE_FLAG_BACKUP_SEMANTICS | flags).open(path)
	}

	fn file_id(file: &File) -> io::Result<(u64, u128)> {
		let mut info = FILE_ID_INFO::default();
		bool_ok(unsafe {
			GetFileInformationByHandleEx(
				file.as_raw_handle() as HANDLE,
				FileIdInfo,
				(&raw mut info).cast(),
				size_of::<FILE_ID_INFO>() as u32,
			)
		})?;

		Ok((info.VolumeSerialNumber, u128::from_le_bytes(info.FileId.Identifier)))
	}
}
