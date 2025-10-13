use serde::{Deserialize, Serialize};

use crate::{ByteStr, Error, ToByteStr};

#[derive(Debug, Deserialize, Serialize)]
pub struct Rename<'a> {
	pub id:   u32,
	pub from: ByteStr<'a>,
	pub to:   ByteStr<'a>,
}

impl<'a> Rename<'a> {
	pub fn new<F, T>(from: F, to: T) -> Result<Self, Error>
	where
		F: ToByteStr<'a>,
		T: ToByteStr<'a>,
	{
		Ok(Self { id: 0, from: from.to_byte_str()?, to: to.to_byte_str()? })
	}

	pub fn len(&self) -> usize { size_of_val(&self.id) + 4 + self.from.len() + 4 + self.to.len() }
}
