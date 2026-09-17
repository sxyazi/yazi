use std::{ffi::{CStr, OsStr}, os::fd::AsFd};
#[cfg(target_os = "netbsd")]
use std::{mem, os::fd::AsRawFd};

#[cfg(not(target_os = "netbsd"))]
use rustix::fs;
#[cfg(target_os = "netbsd")]
use yazi_shim::nonneg_ok;

use super::Casefold;

impl Casefold {
	pub(super) fn case_sensitive(dir: &impl AsFd) -> Option<bool> {
		#[cfg(not(target_os = "netbsd"))]
		let stat = fs::fstatfs(dir).ok()?;

		#[cfg(target_os = "netbsd")]
		let stat = {
			let mut stat = unsafe { mem::zeroed() };
			nonneg_ok(unsafe { libc::fstatvfs(dir.as_fd().as_raw_fd(), &mut stat) }).ok()?;
			stat
		};

		let name = stat.f_fstypename.map(|c| c as u8);
		let name = CStr::from_bytes_until_nul(&name).ok()?.to_bytes();
		matches!(name, b"ext2fs" | b"ffs" | b"mfs" | b"tmpfs" | b"ufs").then_some(true)
	}

	pub(super) fn is_same_mount(dir: &impl AsFd, name: &OsStr) -> bool { false }
}
