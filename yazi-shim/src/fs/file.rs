use std::{fs::File, io, mem, os::windows::io::{AsRawHandle, FromRawHandle}, ptr};

use windows_sys::{Wdk::{Foundation::OBJECT_ATTRIBUTES, Storage::FileSystem::{FILE_OPEN_REPARSE_POINT, NtOpenFile}}, Win32::{Foundation::{ERROR_DELETE_PENDING, ERROR_INVALID_FUNCTION, ERROR_INVALID_PARAMETER, ERROR_NOT_SUPPORTED, OBJ_DONT_REPARSE, RtlNtStatusToDosError, STATUS_DELETE_PENDING, STATUS_PENDING, UNICODE_STRING}, Storage::FileSystem::{FILE_BASIC_INFO, FILE_DISPOSITION_FLAG_DELETE, FILE_DISPOSITION_FLAG_IGNORE_READONLY_ATTRIBUTE, FILE_DISPOSITION_FLAG_POSIX_SEMANTICS, FILE_DISPOSITION_INFO, FILE_DISPOSITION_INFO_EX, FILE_SHARE_DELETE, FILE_SHARE_READ, FILE_SHARE_WRITE, FileBasicInfo, FileDispositionInfo, FileDispositionInfoEx, GetFileInformationByHandleEx, SetFileInformationByHandle}, System::IO::{IO_STATUS_BLOCK, IO_STATUS_BLOCK_0}}};

use crate::bool_ok;

pub trait FileExt {
	fn open_at(&self, name: &[u16], access: u32, options: u32) -> io::Result<File>;

	fn basic_info(&self) -> io::Result<FILE_BASIC_INFO>;

	fn delete(self) -> io::Result<()>;
}

impl FileExt for File {
	fn open_at(&self, name: &[u16], access: u32, options: u32) -> io::Result<File> {
		let len = u16::try_from(mem::size_of_val(name)).map_err(|_| io::ErrorKind::InvalidInput)?;
		// Some third-party NtOpenFile hooks dereference even empty names (rust-lang/rust#143078).
		let buffer = if name.is_empty() { &[0] } else { name };

		let name = UNICODE_STRING {
			Length:        len,
			MaximumLength: mem::size_of_val(buffer) as _,
			Buffer:        buffer.as_ptr().cast_mut(),
		};
		let object = OBJECT_ATTRIBUTES {
			Length: mem::size_of::<OBJECT_ATTRIBUTES>() as _,
			RootDirectory: self.as_raw_handle(),
			ObjectName: &name,
			Attributes: OBJ_DONT_REPARSE,
			..Default::default()
		};

		let mut handle = ptr::null_mut();
		let mut status =
			IO_STATUS_BLOCK { Anonymous: IO_STATUS_BLOCK_0 { Status: STATUS_PENDING }, Information: 0 };
		let result = unsafe {
			NtOpenFile(
				&mut handle,
				access,
				&object,
				&mut status,
				FILE_SHARE_DELETE | FILE_SHARE_READ | FILE_SHARE_WRITE,
				FILE_OPEN_REPARSE_POINT | options,
			)
		};
		if result >= 0 {
			return Ok(unsafe { Self::from_raw_handle(handle) });
		}

		// Preserve pending deletion instead of mapping it to the ambiguous ERROR_ACCESS_DENIED.
		let code = if result == STATUS_DELETE_PENDING {
			ERROR_DELETE_PENDING
		} else {
			unsafe { RtlNtStatusToDosError(result) }
		};
		Err(io::Error::from_raw_os_error(code as _))
	}

	fn basic_info(&self) -> io::Result<FILE_BASIC_INFO> {
		let mut info = FILE_BASIC_INFO::default();
		bool_ok(unsafe {
			GetFileInformationByHandleEx(
				self.as_raw_handle(),
				FileBasicInfo,
				(&raw mut info).cast(),
				mem::size_of_val(&info) as _,
			)
		})?;
		Ok(info)
	}

	fn delete(self) -> io::Result<()> {
		let info = FILE_DISPOSITION_INFO_EX {
			Flags: FILE_DISPOSITION_FLAG_DELETE
				| FILE_DISPOSITION_FLAG_POSIX_SEMANTICS
				| FILE_DISPOSITION_FLAG_IGNORE_READONLY_ATTRIBUTE,
		};
		let result = bool_ok(unsafe {
			SetFileInformationByHandle(
				self.as_raw_handle(),
				FileDispositionInfoEx,
				&info as *const _ as _,
				mem::size_of_val(&info) as _,
			)
		});

		match result {
			Err(e) if let Some(code) = e.raw_os_error() => match code as u32 {
				// The extended class or flags were rejected
				ERROR_INVALID_PARAMETER => {}
				// The filesystem does not implement the extended operation
				ERROR_INVALID_FUNCTION => {}
				// The requested deletion semantics are unsupported
				ERROR_NOT_SUPPORTED => {}
				_ => return Err(e),
			},
			result => return result,
		}

		let info = FILE_DISPOSITION_INFO { DeleteFile: true };
		bool_ok(unsafe {
			SetFileInformationByHandle(
				self.as_raw_handle(),
				FileDispositionInfo,
				&info as *const _ as _,
				mem::size_of_val(&info) as _,
			)
		})
	}
}
