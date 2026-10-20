use std::{fs::File, io, mem, os::windows::io::AsRawHandle, slice};

use windows_sys::Win32::{Foundation::ERROR_NO_MORE_FILES, Storage::FileSystem::{FILE_FULL_DIR_INFO, FileFullDirectoryInfo, GetFileInformationByHandleEx}};

use crate::{bool_ok, fs::DirEntry};

pub struct ReadDir {
	dir:      File,
	buffer:   Vec<u64>,
	offset:   usize,
	name_buf: Vec<u16>,
}

impl ReadDir {
	pub fn new(dir: File) -> io::Result<Self> {
		let mut reader = Self { dir, buffer: vec![0; 8192], offset: 0, name_buf: Vec::new() };
		reader.fill()?;
		Ok(reader)
	}

	pub fn file(&self) -> &File { &self.dir }

	/// Continues reading the same directory handle, retaining entries between calls.
	#[allow(clippy::should_implement_trait)]
	pub fn next(&mut self) -> io::Result<Option<DirEntry<'_>>> {
		if self.buffer.is_empty() {
			return Ok(None);
		} else if self.offset == mem::size_of_val(&*self.buffer) && !self.fill()? {
			return Ok(None);
		}

		// Some filesystem drivers return unaligned entries; keep the trailing name's full provenance.
		let entry =
			unsafe { self.buffer.as_ptr().cast::<u8>().add(self.offset).cast::<FILE_FULL_DIR_INFO>() };
		let (next, attrs, name, length) = unsafe {
			(
				(&raw const (*entry).NextEntryOffset).read_unaligned(),
				(&raw const (*entry).FileAttributes).read_unaligned(),
				(&raw const (*entry).FileName).cast::<u16>(),
				(&raw const (*entry).FileNameLength).read_unaligned() as usize / 2,
			)
		};

		let name = if name.is_aligned() {
			unsafe { slice::from_raw_parts(name, length) }
		} else {
			self.name_buf.clear();
			self.name_buf.extend((0..length).map(|i| unsafe { name.add(i).read_unaligned() }));
			&*self.name_buf
		};

		self.offset =
			if next == 0 { mem::size_of_val(&*self.buffer) } else { self.offset + next as usize };
		Ok(Some(DirEntry { dir: &self.dir, name, attrs }))
	}

	fn fill(&mut self) -> io::Result<bool> {
		let result = bool_ok(unsafe {
			GetFileInformationByHandleEx(
				self.dir.as_raw_handle(),
				FileFullDirectoryInfo,
				self.buffer.as_mut_ptr().cast(),
				mem::size_of_val(&*self.buffer) as _,
			)
		});

		match result {
			Err(e) if e.raw_os_error() == Some(ERROR_NO_MORE_FILES as _) => self.buffer.clear(),
			result => result?,
		}

		self.offset = 0;
		Ok(!self.buffer.is_empty())
	}
}
