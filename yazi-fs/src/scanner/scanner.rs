use std::{ffi::CStr, io, mem, os::fd::AsFd};

use rustix::fs;

pub struct Scanner;

impl Scanner {
	pub fn visit<F>(dir: &impl AsFd, mut visitor: F) -> io::Result<()>
	where
		F: FnMut(&CStr, u64) -> io::Result<bool>,
	{
		#[cfg(any(target_os = "linux", target_os = "android"))]
		{
			let mut buf = [mem::MaybeUninit::uninit(); 64 * 1024 + 8];
			let mut entries = fs::RawDir::new(dir, &mut buf);
			while let Some(entry) = entries.next() {
				let entry = entry?;
				let name = entry.file_name();
				if name != c"." && name != c".." && !visitor(name, entry.ino())? {
					break;
				}
			}
			Ok(())
		}

		#[cfg(not(any(target_os = "linux", target_os = "android")))]
		{
			for entry in fs::Dir::read_from(dir)? {
				let entry = entry?;
				let name = entry.file_name();
				if name != c"." && name != c".." && !visitor(name, entry.ino())? {
					break;
				}
			}
			Ok(())
		}
	}
}
