use std::{io::{self, ErrorKind}, ptr};

use rustix::{fs::{self, Mode}, mm::{self, MapFlags, ProtFlags}, shm::{self, OFlags}};

use super::NamedSharedMemory;

impl Drop for NamedSharedMemory {
	fn drop(&mut self) {
		// The peer unlinks the object after reading it.
		unsafe { mm::munmap(self.ptr, self.len) }.ok();
	}
}

impl NamedSharedMemory {
	pub fn new(data: &[u8]) -> io::Result<Self> {
		if data.is_empty() {
			return Err(io::Error::new(ErrorKind::InvalidInput, "empty shared memory"));
		}

		let name = Self::random_name(b"/yazi-")?;
		let fd =
			shm::open(&name, OFlags::CREATE | OFlags::EXCL | OFlags::RDWR, Mode::RUSR | Mode::WUSR)?;

		// Resize the shared memory object to the size of our data.
		fs::ftruncate(&fd, data.len() as u64).inspect_err(|_| _ = shm::unlink(&name))?;

		// Map the shared memory object into our address space.
		let ptr = unsafe {
			mm::mmap(
				ptr::null_mut(),
				data.len(),
				ProtFlags::READ | ProtFlags::WRITE,
				MapFlags::SHARED,
				&fd,
				0,
			)
		}
		.inspect_err(|_| _ = shm::unlink(&name))?;

		// Copy the data into the shared memory.
		unsafe { ptr::copy_nonoverlapping(data.as_ptr(), ptr.cast(), data.len()) };

		Ok(Self { name, _fd: fd, ptr, len: data.len() })
	}
}
