use std::io;

pub struct NamedSharedMemory {
	pub name:          Vec<u8>,
	#[cfg(any(unix, windows))]
	pub(super) ptr:    *mut std::ffi::c_void,
	#[cfg(unix)]
	pub(super) len:    usize,
	#[cfg(unix)]
	pub(super) _fd:    std::os::fd::OwnedFd,
	#[cfg(windows)]
	pub(super) handle: *mut std::ffi::c_void,
}

// SAFETY: This value only uses the pointer to clean up the mapping in `Drop`;
// it never reads or writes through it, so moving the value to another thread is
// safe.
#[cfg(any(unix, windows))]
unsafe impl Send for NamedSharedMemory {}

impl NamedSharedMemory {
	#[cfg(any(unix, windows))]
	pub(super) fn random_name(prefix: &[u8]) -> io::Result<Vec<u8>> {
		use data_encoding::BASE32_NOPAD;
		use rand::{TryRng, rngs::SysRng};

		let mut bytes = [0; 15];
		SysRng.try_fill_bytes(&mut bytes)?;

		let mut name = Vec::with_capacity(prefix.len() + 24);
		name.extend_from_slice(prefix);
		name.extend_from_slice(BASE32_NOPAD.encode_mut_str(&bytes, &mut [0; 24]).as_bytes());

		Ok(name)
	}
}
