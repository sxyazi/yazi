use std::{io::{self, ErrorKind}, ptr};

use windows_sys::Win32::{Foundation::{CloseHandle, INVALID_HANDLE_VALUE}, System::Memory::{CreateFileMappingW, FILE_MAP_WRITE, MEMORY_MAPPED_VIEW_ADDRESS, MapViewOfFile, PAGE_READWRITE, UnmapViewOfFile}};

use super::NamedSharedMemory;

impl Drop for NamedSharedMemory {
	fn drop(&mut self) {
		unsafe {
			UnmapViewOfFile(MEMORY_MAPPED_VIEW_ADDRESS { Value: self.ptr });
			CloseHandle(self.handle);
		}
	}
}

impl NamedSharedMemory {
	pub fn new(data: &[u8]) -> io::Result<Self> {
		if data.is_empty() {
			return Err(io::Error::new(ErrorKind::InvalidInput, "empty shared memory"));
		}

		let name = Self::random_name(b"yazi-")?;
		let wide: Vec<u16> = name.iter().map(|&b| b as u16).chain([0]).collect();

		let size = data.len() as u64;
		let handle = unsafe {
			CreateFileMappingW(
				INVALID_HANDLE_VALUE,
				ptr::null(),
				PAGE_READWRITE,
				(size >> 32) as u32,
				size as u32,
				wide.as_ptr(),
			)
		};
		if handle.is_null() {
			return Err(io::Error::last_os_error());
		}

		let view = unsafe { MapViewOfFile(handle, FILE_MAP_WRITE, 0, 0, data.len()) };
		if view.Value.is_null() {
			let err = io::Error::last_os_error();
			unsafe { CloseHandle(handle) };
			return Err(err);
		}

		unsafe { ptr::copy_nonoverlapping(data.as_ptr(), view.Value.cast(), data.len()) };
		Ok(Self { name, ptr: view.Value, handle })
	}
}
