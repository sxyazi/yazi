use std::{fs::Metadata, time::SystemTime};

use bitflags::bitflags;

bitflags! {
	#[derive(Clone, Copy, Debug, Default)]
	pub struct ChaMeta: u8 {
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
	pub meta:        ChaMeta,
	pub len:         u64,
	pub accessed:    Option<SystemTime>,
	pub created:     Option<SystemTime>,
	pub modified:    Option<SystemTime>,
	#[cfg(unix)]
	pub permissions: u32,
}

impl From<Metadata> for Cha {
	fn from(m: Metadata) -> Self {
		let mut cm = ChaMeta::empty();
		if m.is_dir() {
			cm |= ChaMeta::DIR;
		}

		#[cfg(unix)]
		{
			use std::os::unix::prelude::FileTypeExt;
			if m.file_type().is_block_device() {
				cm |= ChaMeta::BLOCK_DEVICE;
			}
			if m.file_type().is_char_device() {
				cm |= ChaMeta::CHAR_DEVICE;
			}
			if m.file_type().is_fifo() {
				cm |= ChaMeta::FIFO;
			}
			if m.file_type().is_socket() {
				cm |= ChaMeta::SOCKET;
			}
		}

		Self {
			meta:     cm,
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
	pub fn with_meta(mut self, meta: ChaMeta) -> Self {
		self.meta |= meta;
		self
	}
}

impl Cha {
	#[inline]
	pub fn is_dir(self) -> bool { self.meta.contains(ChaMeta::DIR) }

	#[inline]
	pub fn is_hidden(self) -> bool { self.meta.contains(ChaMeta::HIDDEN) }

	#[inline]
	pub fn is_link(self) -> bool { self.meta.contains(ChaMeta::LINK) }

	#[inline]
	pub fn is_bad_link(self) -> bool { self.meta.contains(ChaMeta::BAD_LINK) }

	#[cfg(unix)]
	#[inline]
	pub fn is_block_device(self) -> bool { self.meta.contains(ChaMeta::BLOCK_DEVICE) }

	#[cfg(unix)]
	#[inline]
	pub fn is_char_device(self) -> bool { self.meta.contains(ChaMeta::CHAR_DEVICE) }

	#[cfg(unix)]
	#[inline]
	pub fn is_fifo(self) -> bool { self.meta.contains(ChaMeta::FIFO) }

	#[cfg(unix)]
	#[inline]
	pub fn is_socket(self) -> bool { self.meta.contains(ChaMeta::SOCKET) }
}
