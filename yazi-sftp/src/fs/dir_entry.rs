use std::{borrow::Cow, ffi::OsStr, path::PathBuf};

use crate::{ByteStr, fs::Attrs};

pub struct DirEntry<'a> {
	pub(super) dir:       ByteStr<'a>,
	pub(super) name:      ByteStr<'a>,
	pub(super) long_name: ByteStr<'a>,
	pub(super) attrs:     Attrs,
}

impl<'a> DirEntry<'a> {
	#[must_use]
	pub fn path(&self) -> PathBuf { self.dir.join(&self.name) }

	#[must_use]
	pub fn name(&self) -> Cow<'_, OsStr> { self.name.to_os_str() }

	#[must_use]
	pub fn long_name(&self) -> Cow<'_, OsStr> { self.long_name.to_os_str() }

	pub fn attrs(&self) -> &Attrs { &self.attrs }

	pub fn nlink(&self) -> Option<u64> { str::from_utf8(self.long_name_field(1)?).ok()?.parse().ok() }

	pub fn user(&self) -> Option<Cow<'_, OsStr>> {
		let b = self.long_name_field(2)?;
		Some(unsafe {
			match ByteStr::from_str_bytes_unchecked(b).to_os_str() {
				Cow::Borrowed(_) => OsStr::from_encoded_bytes_unchecked(b).into(),
				Cow::Owned(s) => s.into(),
			}
		})
	}

	pub fn group(&self) -> Option<Cow<'_, OsStr>> {
		let b = self.long_name_field(3)?;
		Some(unsafe {
			match ByteStr::from_str_bytes_unchecked(b).to_os_str() {
				Cow::Borrowed(_) => OsStr::from_encoded_bytes_unchecked(b).into(),
				Cow::Owned(s) => s.into(),
			}
		})
	}

	fn long_name_field(&self, n: usize) -> Option<&[u8]> {
		self.long_name.split(|b| b.is_ascii_whitespace()).filter(|s| !s.is_empty()).nth(n)
	}
}
