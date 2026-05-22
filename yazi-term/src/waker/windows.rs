use std::{io, ops::Deref, os::windows::io::{AsRawHandle, FromRawHandle, OwnedHandle}, ptr, sync::Arc};

use windows_sys::Win32::System::Threading;

#[derive(Debug)]
pub(crate) struct Waker {
	handle: Arc<OwnedHandle>,
}

impl Deref for Waker {
	type Target = OwnedHandle;

	fn deref(&self) -> &Self::Target { &self.handle }
}

impl Waker {
	pub(crate) fn new() -> io::Result<Self> {
		let handle = unsafe { Threading::CreateEventW(ptr::null(), 0, 0, ptr::null()) };
		if handle.is_null() {
			Err(io::Error::last_os_error())
		} else {
			Ok(Self { handle: Arc::new(unsafe { OwnedHandle::from_raw_handle(handle) }) })
		}
	}

	pub fn wake(&self) -> io::Result<()> {
		if unsafe { Threading::SetEvent(self.handle.as_raw_handle()) } == 0 {
			Err(io::Error::last_os_error())
		} else {
			Ok(())
		}
	}
}
