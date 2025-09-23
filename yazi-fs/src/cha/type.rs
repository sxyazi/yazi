use std::fs::FileType;

use crate::cha::ChaMode;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ChaType {
	File,
	Dir,
	Link,
	Block,
	Char,
	Sock,
	FIFO,
	Unknown,
}

impl From<ChaMode> for ChaType {
	fn from(value: ChaMode) -> Self { *value }
}

impl From<FileType> for ChaType {
	fn from(value: FileType) -> Self {
		#[cfg(unix)]
		{
			use std::os::unix::fs::FileTypeExt;
			if value.is_file() {
				Self::File
			} else if value.is_dir() {
				Self::Dir
			} else if value.is_symlink() {
				Self::Link
			} else if value.is_block_device() {
				Self::Block
			} else if value.is_char_device() {
				Self::Char
			} else if value.is_socket() {
				Self::Sock
			} else if value.is_fifo() {
				Self::FIFO
			} else {
				Self::Unknown
			}
		}
		#[cfg(windows)]
		{
			if value.is_file() {
				Self::File
			} else if value.is_dir() {
				Self::Dir
			} else if value.is_symlink() {
				Self::Link
			} else {
				Self::Unknown
			}
		}
	}
}

impl ChaType {
	#[inline]
	pub fn is_file(self) -> bool { self == Self::File }

	#[inline]
	pub fn is_dir(self) -> bool { self == Self::Dir }

	#[inline]
	pub fn is_block(self) -> bool { self == Self::Block }

	#[inline]
	pub fn is_char(self) -> bool { self == Self::Char }

	#[inline]
	pub fn is_sock(self) -> bool { self == Self::Sock }

	#[inline]
	pub fn is_fifo(self) -> bool { self == Self::FIFO }
}
