use std::{fs::Metadata, time::SystemTime};

use bitflags::bitflags;

bitflags! {
	#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
	pub struct ChaKind: u8 {
		const DIR           = 0b00000001;

		const HIDDEN        = 0b00000010;
		const LINK          = 0b00000100;
		const BAD_LINK      = 0b00001000;

		const BLOCK_DEVICE  = 0b00010000;
		const CHAR_DEVICE   = 0b00100000;
		const FIFO          = 0b01000000;
		const SOCKET        = 0b10000000;
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
	pub permissions: u32,
}

impl From<Metadata> for Cha {
	fn from(m: Metadata) -> Self {
		let mut ck = ChaKind::empty();
		if m.is_dir() {
			ck |= ChaKind::DIR;
		} else if m.is_symlink() {
			ck |= ChaKind::LINK;
		}

		#[cfg(unix)]
		{
			use std::os::unix::prelude::FileTypeExt;
			if m.file_type().is_block_device() {
				ck |= ChaKind::BLOCK_DEVICE;
			}
			if m.file_type().is_char_device() {
				ck |= ChaKind::CHAR_DEVICE;
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
			// TODO: remove this once https://github.com/rust-lang/rust/issues/108277 is fixed.
			created:  None,
			modified: m.modified().ok(),

			#[cfg(unix)]
			permissions:              {
				use std::os::unix::prelude::PermissionsExt;
				m.permissions().mode()
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
	pub fn is_dir(self) -> bool { self.kind.contains(ChaKind::DIR) }

	#[inline]
	pub fn is_hidden(self) -> bool { self.kind.contains(ChaKind::HIDDEN) }

	#[inline]
	pub fn is_link(self) -> bool { self.kind.contains(ChaKind::LINK) }

	#[inline]
	pub fn is_bad_link(self) -> bool { self.kind.contains(ChaKind::BAD_LINK) }

	#[cfg(unix)]
	#[inline]
	pub fn is_block_device(self) -> bool { self.kind.contains(ChaKind::BLOCK_DEVICE) }

	#[cfg(unix)]
	#[inline]
	pub fn is_char_device(self) -> bool { self.kind.contains(ChaKind::CHAR_DEVICE) }

	#[cfg(unix)]
	#[inline]
	pub fn is_fifo(self) -> bool { self.kind.contains(ChaKind::FIFO) }

	#[cfg(unix)]
	#[inline]
	pub fn is_socket(self) -> bool { self.kind.contains(ChaKind::SOCKET) }
}
