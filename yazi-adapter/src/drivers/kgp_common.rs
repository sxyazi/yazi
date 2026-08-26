use std::{io::{self, Write}, ops::Deref};

use base64::{Engine, engine::general_purpose};
use yazi_ffi::shm::NamedSharedMemory;
use yazi_shim::cell::SyncCell;

pub(super) fn kgp_id() -> u32 {
	static CACHE: SyncCell<Option<u32>> = SyncCell::new(None);
	match CACHE.get() {
		Some(n) => n,
		None => {
			let n = std::process::id() % (0xffffff + 1);
			CACHE.set(Some(n));
			n
		}
	}
}

// --- KgpPayload
pub(super) struct KgpPayload {
	bytes: Vec<u8>,
	_shm:  Option<NamedSharedMemory>,
}

impl Deref for KgpPayload {
	type Target = [u8];

	fn deref(&self) -> &Self::Target { &self.bytes }
}

impl KgpPayload {
	pub(super) fn new(cap: usize) -> Self { Self { bytes: Vec::with_capacity(cap), _shm: None } }

	pub(super) fn with(cap: usize, shm: NamedSharedMemory) -> Self {
		Self { bytes: Vec::with_capacity(cap), _shm: Some(shm) }
	}

	pub(super) fn name(&self) -> String {
		let Some(shm) = &self._shm else { return String::new() };
		general_purpose::STANDARD.encode(&shm.name)
	}
}

impl Write for KgpPayload {
	fn write(&mut self, buf: &[u8]) -> io::Result<usize> { self.bytes.write(buf) }

	fn flush(&mut self) -> io::Result<()> { self.bytes.flush() }
}
