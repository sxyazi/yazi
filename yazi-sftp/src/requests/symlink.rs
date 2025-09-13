use serde::{Deserialize, Serialize};

use crate::ByteStr;

#[derive(Debug, Deserialize, Serialize)]
pub struct Symlink<'a> {
	pub id:       u32,
	pub link:     ByteStr<'a>,
	pub original: ByteStr<'a>,
}

impl<'a> Symlink<'a> {
	pub fn new<L, O>(link: L, original: O) -> Self
	where
		L: Into<ByteStr<'a>>,
		O: Into<ByteStr<'a>>,
	{
		Self { id: 0, link: link.into(), original: original.into() }
	}

	pub fn len(&self) -> usize {
		size_of_val(&self.id) + 4 + self.link.len() + 4 + self.original.len()
	}
}
