use serde::{Deserialize, Serialize};

use crate::{ByteStr, Error, ToByteStr};

#[derive(Debug, Deserialize, Serialize)]
pub struct Stat<'a> {
	pub id:   u32,
	pub path: ByteStr<'a>,
}

impl<'a> Stat<'a> {
	pub fn new<P>(path: P) -> Result<Self, Error>
	where
		P: ToByteStr<'a>,
	{
		Ok(Self { id: 0, path: path.to_byte_str()? })
	}

	pub fn len(&self) -> usize { size_of_val(&self.id) + 4 + self.path.len() }
}
