use std::{fs::Metadata, time::SystemTime};

use bitflags::bitflags;

bitflags! {
	#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
	pub struct ChaKind: u8 {
		const DIR    = 0b00000001;

		const HIDDEN = 0b00000010;
		const LINK   = 0b00000100;
		const ORPHAN = 0b00001000;

		const BLOCK  = 0b00010000;
		const CHAR   = 0b00100000;
		const FIFO   = 0b01000000;
		const SOCKET = 0b10000000;
	}
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Cha {
	pub kind:        ChaKind,
	pub len:         u64,
	pub accessed:    Option<SystemTime>,
	pub created:     Option<SystemTime>,
	pub modified:    Option<SystemTime>,
	#[cfg(unix)]
	pub permissions: libc::mode_t,
	#[cfg(unix)]
	pub uid:         libc::uid_t,
	#[cfg(unix)]
	pub gid:         libc::gid_t,
	#[cfg(unix)]
	pub nlink:       libc::nlink_t,
}

impl From<Metadata> for Cha {
	fn from(m: Metadata) -> Self {
		let mut ck = ChaKind::empty();
		if m.is_dir() {
			ck |= ChaKind::DIR;
		}

		#[cfg(unix)]
		{
			use std::os::unix::prelude::FileTypeExt;
			if m.file_type().is_block_device() {
				ck |= ChaKind::BLOCK;
			}
			if m.file_type().is_char_device() {
				ck |= ChaKind::CHAR;
			}
			if m.file_type().is_fifo() {
				ck |= ChaKind::FIFO;
			}
			if m.file_type().is_socket() {
				ck |= ChaKind::SOCKET;
			}
		}

		Self {
			kind:     ck,
			len:      m.len(),
			accessed: m.accessed().ok(),
			created:  m.created().ok(),
			modified: m.modified().ok(),

			#[cfg(unix)]
			permissions:              {
				use std::os::unix::prelude::PermissionsExt;
				m.permissions().mode() as _
			},
			#[cfg(unix)]
			uid:                      {
				use std::os::unix::fs::MetadataExt;
				m.uid() as _
			},
			#[cfg(unix)]
			gid:                      {
				use std::os::unix::fs::MetadataExt;
				m.gid() as _
			},
			#[cfg(unix)]
			nlink:                    {
				use std::os::unix::fs::MetadataExt;
				m.nlink() as _
			},
		}
	}
}

impl Cha {
	#[inline]
	pub fn with_kind(mut self, kind: ChaKind) -> Self {
		self.kind |= kind;
		self
	}
}

impl Cha {
	#[inline]
	pub fn is_dir(&self) -> bool { self.kind.contains(ChaKind::DIR) }

	#[inline]
	pub fn is_hidden(&self) -> bool { self.kind.contains(ChaKind::HIDDEN) }

	#[inline]
	pub fn is_link(&self) -> bool { self.kind.contains(ChaKind::LINK) }

	#[inline]
	pub fn is_orphan(&self) -> bool { self.kind.contains(ChaKind::ORPHAN) }

	#[inline]
	pub fn is_block(&self) -> bool { self.kind.contains(ChaKind::BLOCK) }

	#[inline]
	pub fn is_char(&self) -> bool { self.kind.contains(ChaKind::CHAR) }

	#[inline]
	pub fn is_fifo(&self) -> bool { self.kind.contains(ChaKind::FIFO) }

	#[inline]
	pub fn is_sock(&self) -> bool { self.kind.contains(ChaKind::SOCKET) }

	#[inline]
	pub fn is_exec(&self) -> bool {
		#[cfg(unix)]
		{
			self.permissions & libc::S_IXUSR != 0
		}
		#[cfg(windows)]
		{
			false
		}
	}

	#[inline]
	pub fn is_sticky(&self) -> bool {
		#[cfg(unix)]
		{
			self.permissions & libc::S_ISVTX != 0
		}
		#[cfg(windows)]
		{
			false
		}
	}
}
