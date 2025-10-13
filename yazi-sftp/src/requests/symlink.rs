use serde::{Deserialize, Serialize};

use crate::{ByteStr, Error, ToByteStr};

#[derive(Debug, Deserialize, Serialize)]
pub struct Symlink<'a> {
	pub id:       u32,
	pub link:     ByteStr<'a>,
	pub original: ByteStr<'a>,
}

impl<'a> Symlink<'a> {
	pub fn new<L, O>(link: L, original: O) -> Result<Self, Error>
	where
		L: ToByteStr<'a>,
		O: ToByteStr<'a>,
	{
		Ok(Self { id: 0, link: link.to_byte_str()?, original: original.to_byte_str()? })
	}

	pub fn len(&self) -> usize {
		size_of_val(&self.id) + 4 + self.link.len() + 4 + self.original.len()
	}
}
