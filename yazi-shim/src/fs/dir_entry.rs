use std::fs::File;

use windows_sys::Win32::Storage::FileSystem::{FILE_ATTRIBUTE_DIRECTORY, FILE_ATTRIBUTE_REPARSE_POINT};

pub struct DirEntry<'a> {
	pub dir:   &'a File,
	pub name:  &'a [u16],
	pub attrs: u32,
}

impl DirEntry<'_> {
	pub fn is_dir(&self) -> bool { self.attrs & FILE_ATTRIBUTE_DIRECTORY != 0 }

	pub fn is_reparse(&self) -> bool { self.attrs & FILE_ATTRIBUTE_REPARSE_POINT != 0 }
}
