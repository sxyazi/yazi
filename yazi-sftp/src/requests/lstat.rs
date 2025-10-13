use serde::{Deserialize, Serialize};

use crate::{ByteStr, Error, ToByteStr};

#[derive(Debug, Deserialize, Serialize)]
pub struct Lstat<'a> {
	pub id:   u32,
	pub path: ByteStr<'a>,
}

impl Lstat<'_> {
	pub fn new<'a, P>(path: P) -> Result<Lstat<'a>, Error>
	where
		P: ToByteStr<'a>,
	{
		Ok(Lstat { id: 0, path: path.to_byte_str()? })
	}

	pub fn len(&self) -> usize { size_of_val(&self.id) + 4 + self.path.len() }
}
