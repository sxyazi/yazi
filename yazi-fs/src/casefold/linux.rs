use std::{ffi::OsStr, os::fd::AsFd};

use libc::{BTRFS_SUPER_MAGIC, EXT4_SUPER_MAGIC, F2FS_SUPER_MAGIC, TMPFS_MAGIC};
use rustix::fs::{AtFlags, StatxFlags, fstatfs, ioctl_getflags, statx};

use super::Casefold;

impl Casefold {
	pub(super) fn case_sensitive(dir: &impl AsFd) -> Option<bool> {
		const FS_CASEFOLD_FL: u32 = 0x4000_0000;

		let fs = fstatfs(dir).ok()?;
		if fs.f_type == BTRFS_SUPER_MAGIC as _
			|| fs.f_type == EXT4_SUPER_MAGIC as _
			|| fs.f_type == F2FS_SUPER_MAGIC as _
			|| fs.f_type == TMPFS_MAGIC as _
		{
			ioctl_getflags(dir).ok().map(|flags| flags.bits() & FS_CASEFOLD_FL == 0)
		} else {
			None
		}
	}

	pub(super) fn is_same_mount(dir: &impl AsFd, name: &OsStr) -> bool {
		let parent = statx(dir, c"", AtFlags::EMPTY_PATH, StatxFlags::MNT_ID);
		let entry = statx(dir, name, AtFlags::SYMLINK_NOFOLLOW, StatxFlags::MNT_ID);

		let (Ok(p), Ok(e)) = (parent, entry) else { return false };
		p.stx_mnt_id == e.stx_mnt_id && p.stx_mask & e.stx_mask & StatxFlags::MNT_ID.bits() != 0
	}
}
