use std::{fs::{FileType, Metadata}, time::SystemTime};

use bitflags::bitflags;
use yazi_macro::unix_either;

bitflags! {
	#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
	pub struct ChaKind: u8 {
		const DIR    = 0b00000001;

		const HIDDEN = 0b00000010;
		const LINK   = 0b00000100;
		const ORPHAN = 0b00001000;

		const DUMMY  = 0b00010000;
	}
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Cha {
	pub kind:  ChaKind,
	pub len:   u64,
	pub atime: Option<SystemTime>,
	pub btime: Option<SystemTime>,
	#[cfg(unix)]
	pub ctime: Option<SystemTime>,
	pub mtime: Option<SystemTime>,
	#[cfg(unix)]
	pub perm:  libc::mode_t,
	#[cfg(unix)]
	pub uid:   libc::uid_t,
	#[cfg(unix)]
	pub gid:   libc::gid_t,
	#[cfg(unix)]
	pub nlink: libc::nlink_t,
}

impl From<Metadata> for Cha {
	fn from(m: Metadata) -> Self {
		let mut ck = ChaKind::empty();
		if m.is_dir() {
			ck |= ChaKind::DIR;
		}

		Self {
			kind:               ck,
			len:                m.len(),
			atime:              m.accessed().ok(),
			btime:              m.created().ok(),
			#[cfg(unix)]
			ctime:              {
				use std::{os::unix::fs::MetadataExt, time::{Duration, UNIX_EPOCH}};
				UNIX_EPOCH.checked_add(Duration::new(m.ctime() as u64, m.ctime_nsec() as u32))
			},
			mtime:              m.modified().ok(),

			#[cfg(unix)]
			perm:               {
				use std::os::unix::prelude::PermissionsExt;
				m.permissions().mode() as _
			},
			#[cfg(unix)]
			uid:                {
				use std::os::unix::fs::MetadataExt;
				m.uid() as _
			},
			#[cfg(unix)]
			gid:                {
				use std::os::unix::fs::MetadataExt;
				m.gid() as _
			},
			#[cfg(unix)]
			nlink:              {
				use std::os::unix::fs::MetadataExt;
				m.nlink() as _
			},
		}
	}
}

impl From<FileType> for Cha {
	fn from(t: FileType) -> Self {
		let mut kind = ChaKind::DUMMY;

		#[cfg(unix)]
		let perm = {
			use std::os::unix::fs::FileTypeExt;
			if t.is_dir() {
				kind |= ChaKind::DIR;
				libc::S_IFDIR
			} else if t.is_symlink() {
				kind |= ChaKind::LINK;
				libc::S_IFLNK
			} else if t.is_block_device() {
				libc::S_IFBLK
			} else if t.is_char_device() {
				libc::S_IFCHR
			} else if t.is_fifo() {
				libc::S_IFIFO
			} else if t.is_socket() {
				libc::S_IFSOCK
			} else {
				0
			}
		};

		#[cfg(windows)]
		{
			if t.is_dir() {
				kind |= ChaKind::DIR;
			} else if t.is_symlink() {
				kind |= ChaKind::LINK;
			}
		}

		Self {
			kind,
			#[cfg(unix)]
			perm,
			..Default::default()
		}
	}
}

impl Cha {
	#[inline]
	pub fn dummy() -> Self { Self { kind: ChaKind::DUMMY, ..Default::default() } }

	#[inline]
	pub fn with_kind(mut self, kind: ChaKind) -> Self {
		self.kind |= kind;
		self
	}

	#[inline]
	pub fn hits(self, c: Self) -> bool {
		self.len == c.len
			&& self.mtime == c.mtime
			&& unix_either!(self.ctime == c.ctime, true)
			&& self.btime == c.btime
			&& self.kind == c.kind
			&& unix_either!(self.perm == c.perm, true)
	}
}

impl Cha {
	#[inline]
	pub const fn is_dir(&self) -> bool { self.kind.contains(ChaKind::DIR) }

	#[inline]
	pub const fn is_hidden(&self) -> bool { self.kind.contains(ChaKind::HIDDEN) }

	#[inline]
	pub const fn is_link(&self) -> bool { self.kind.contains(ChaKind::LINK) }

	#[inline]
	pub const fn is_orphan(&self) -> bool { self.kind.contains(ChaKind::ORPHAN) }

	#[inline]
	pub const fn is_dummy(&self) -> bool { self.kind.contains(ChaKind::DUMMY) }

	#[inline]
	pub const fn is_block(&self) -> bool {
		unix_either!(self.perm & libc::S_IFMT == libc::S_IFBLK, false)
	}

	#[inline]
	pub const fn is_char(&self) -> bool {
		unix_either!(self.perm & libc::S_IFMT == libc::S_IFCHR, false)
	}

	#[inline]
	pub const fn is_fifo(&self) -> bool {
		unix_either!(self.perm & libc::S_IFMT == libc::S_IFIFO, false)
	}

	#[inline]
	pub const fn is_sock(&self) -> bool {
		unix_either!(self.perm & libc::S_IFMT == libc::S_IFSOCK, false)
	}

	#[inline]
	pub const fn is_exec(&self) -> bool { unix_either!(self.perm & libc::S_IXUSR != 0, false) }

	#[inline]
	pub const fn is_sticky(&self) -> bool { unix_either!(self.perm & libc::S_ISVTX != 0, false) }
}
