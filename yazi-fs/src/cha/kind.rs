use std::fs::Metadata;

use bitflags::bitflags;
use serde::{Deserialize, Serialize};
use yazi_shared::strand::AsStrand;

bitflags! {
	#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Eq, Serialize)]
	pub struct ChaKind: u8 {
		const FOLLOW  = 0b0000_0001;
		const HIDDEN  = 0b0000_0010;
		const SYSTEM  = 0b0000_0100;
		const DUMMY   = 0b0000_1000;
		const REPARSE = 0b0001_0000;
	}
}

impl ChaKind {
	#[inline]
	pub(super) fn hidden<T>(_name: T, _meta: &Metadata) -> Self
	where
		T: AsStrand,
	{
		let mut me = Self::empty();

		#[cfg(unix)]
		{
			if _name.as_strand().starts_with(".") {
				me |= Self::HIDDEN;
			}
		}
		#[cfg(windows)]
		{
			use std::os::windows::fs::MetadataExt;

			use windows_sys::Win32::Storage::FileSystem::{FILE_ATTRIBUTE_HIDDEN, FILE_ATTRIBUTE_SYSTEM};
			if _meta.file_attributes() & FILE_ATTRIBUTE_HIDDEN != 0 {
				me |= Self::HIDDEN;
			}
			if _meta.file_attributes() & FILE_ATTRIBUTE_SYSTEM != 0 {
				me |= Self::SYSTEM;
			}
		}

		me
	}

	#[inline]
	pub(super) fn reparse(_meta: &Metadata) -> Self {
		#[cfg(windows)]
		{
			use std::os::windows::fs::MetadataExt;

			use windows_sys::Win32::Storage::FileSystem::FILE_ATTRIBUTE_REPARSE_POINT;

			if _meta.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0 {
				return Self::REPARSE;
			}
		}

		Self::empty()
	}
}
