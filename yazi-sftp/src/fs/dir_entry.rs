use std::sync::Arc;

use typed_path::UnixPathBuf;

use crate::fs::Attrs;

pub struct DirEntry {
	pub(super) dir:       Arc<typed_path::UnixPathBuf>,
	pub(super) name:      Vec<u8>,
	pub(super) long_name: Vec<u8>,
	pub(super) attrs:     Attrs,
}

impl DirEntry {
	#[must_use]
	pub fn path(&self) -> UnixPathBuf { self.dir.join(&self.name) }

	pub fn name(&self) -> &[u8] { &self.name }

	pub fn long_name(&self) -> &[u8] { &self.long_name }

	pub fn attrs(&self) -> &Attrs { &self.attrs }

	pub fn nlink(&self) -> Option<u64> { str::from_utf8(self.long_name_field(1)?).ok()?.parse().ok() }

	pub fn user(&self) -> Option<&[u8]> { self.long_name_field(2) }

	pub fn group(&self) -> Option<&[u8]> { self.long_name_field(3) }

	fn long_name_field(&self, n: usize) -> Option<&[u8]> {
		self.long_name.split(|b| b.is_ascii_whitespace()).filter(|s| !s.is_empty()).nth(n)
	}
}
