use std::{io, ops::Deref};

use yazi_shim::nonneg_ok;

#[repr(transparent)]
pub struct Copystate(libc::copyfile_state_t);

// The state owns heap resources without thread affinity; ownership may move, but access must stay exclusive.
unsafe impl Send for Copystate {}

impl Deref for Copystate {
	type Target = libc::copyfile_state_t;

	fn deref(&self) -> &Self::Target { &self.0 }
}

impl Drop for Copystate {
	fn drop(&mut self) { unsafe { _ = libc::copyfile_state_free(self.0) } }
}

impl Copystate {
	pub fn new() -> io::Result<Self> {
		let state = unsafe { libc::copyfile_state_alloc() };
		if state.is_null() {
			return Err(io::Error::last_os_error());
		}

		Ok(Self(state))
	}

	pub fn copied(&self) -> io::Result<u64> {
		let mut copied: libc::off_t = 0;
		nonneg_ok(unsafe {
			libc::copyfile_state_get(self.0, libc::COPYFILE_STATE_COPIED as _, (&raw mut copied).cast())
		})?;

		Ok(copied as u64)
	}
}
