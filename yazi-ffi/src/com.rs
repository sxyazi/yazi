use std::io;

use windows::Win32::System::Com::{COINIT_APARTMENTTHREADED, CoInitializeEx, CoUninitialize};

pub struct Com;

impl Com {
	pub fn new() -> io::Result<Self> {
		unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED).ok().map_err(io::Error::other)? };
		Ok(Self)
	}
}

impl Drop for Com {
	fn drop(&mut self) { unsafe { CoUninitialize() } }
}
