use std::{ffi::OsStr, io, mem::size_of, os::unix::ffi::OsStrExt, ptr, slice};

#[cfg(not(target_os = "netbsd"))]
use libc::{getfsstat as getstat, statfs as Stat};
#[cfg(target_os = "netbsd")]
use libc::{getvfsstat as getstat, statvfs as Stat};
use yazi_shim::nonneg_ok;

use super::{Partition, Partitions};

impl Partitions {
	pub(crate) fn mounts() -> io::Result<Self> {
		fn os_str(bytes: &[libc::c_char]) -> &OsStr {
			let bytes = unsafe { slice::from_raw_parts(bytes.as_ptr().cast(), bytes.len()) };
			let end = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
			OsStr::from_bytes(&bytes[..end])
		}

		let entries = loop {
			let count = nonneg_ok(unsafe { getstat(ptr::null_mut(), 0, libc::MNT_WAIT) })?;
			let mut entries: Vec<Stat> = Vec::with_capacity(count as usize + 1);

			let size = entries.capacity() * size_of::<Stat>();
			let count =
				nonneg_ok(unsafe { getstat(entries.as_mut_ptr(), size as _, libc::MNT_WAIT) })? as usize;

			if count >= entries.capacity() {
				continue;
			}

			unsafe { entries.set_len(count) };
			break entries;
		};

		Ok(
			entries
				.iter()
				.filter(|entry| entry.f_mntfromname[0] != 0 && entry.f_mntonname[0] != 0)
				.map(|entry| Partition {
					src: os_str(&entry.f_mntfromname).into(),
					dist: Some(os_str(&entry.f_mntonname).into()),
					fstype: Some(os_str(&entry.f_fstypename).into()),
					..Default::default()
				})
				.collect(),
		)
	}
}
