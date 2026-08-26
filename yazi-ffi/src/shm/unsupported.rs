use std::io::{self, ErrorKind};

use super::NamedSharedMemory;

impl NamedSharedMemory {
	pub fn new(_data: &[u8]) -> io::Result<Self> {
		Err(io::Error::new(ErrorKind::Unsupported, "shared memory is unsupported on this platform"))
	}
}
