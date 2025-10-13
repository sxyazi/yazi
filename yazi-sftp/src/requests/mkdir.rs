use serde::{Deserialize, Serialize};

use crate::{ByteStr, Error, ToByteStr, fs::Attrs};

#[derive(Debug, Deserialize, Serialize)]
pub struct Mkdir<'a> {
	pub id:    u32,
	pub path:  ByteStr<'a>,
	pub attrs: Attrs,
}

impl<'a> Mkdir<'a> {
	pub fn new<P>(path: P, attrs: Attrs) -> Result<Self, Error>
	where
		P: ToByteStr<'a>,
	{
		Ok(Self { id: 0, path: path.to_byte_str()?, attrs })
	}

	pub fn len(&self) -> usize { size_of_val(&self.id) + 4 + self.path.len() + self.attrs.len() }
}
