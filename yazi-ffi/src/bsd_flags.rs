use std::{fs::File, io, os::{fd::AsRawFd, macos::fs::MetadataExt}};

use yazi_shim::nonneg_ok;

// XNU's fsioc_cas_bsdflags layout, used by FSIOC_CAS_BSDFLAGS.
#[repr(C)]
pub struct BsdFlags {
	expected: u32,
	new:      u32,
	actual:   u32,
}

impl BsdFlags {
	pub fn set(file: &File, flags: u32, preserve: u32) -> io::Result<()> {
		let mut current = file.metadata()?.st_flags();

		// UF_COMPRESSED requires an atomic update; retry races before using Apple's fchflags fallback.
		for _ in 0..4 {
			let mut cas =
				Self { expected: current, new: flags | (current & preserve), actual: u32::MAX };

			let result = unsafe {
				ffsctl(file.as_raw_fd(), libc::_IOWR::<Self>(b'A' as _, 20), (&raw mut cas).cast(), 0)
			};

			if result == 0 && cas.actual == current {
				return Ok(());
			} else if result == 0 {
				current = cas.actual;
			} else if io::Error::last_os_error().raw_os_error() != Some(libc::EAGAIN) {
				break;
			}
		}

		nonneg_ok(unsafe { libc::fchflags(file.as_raw_fd(), flags | (current & preserve)) })
	}
}

unsafe extern "C" {
	fn ffsctl(
		fd: libc::c_int,
		request: libc::c_ulong,
		data: *mut libc::c_void,
		options: libc::c_uint,
	) -> libc::c_int;
}
