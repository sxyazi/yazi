use std::borrow::Cow;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Write<'a> {
	pub id:     u32,
	pub handle: Cow<'a, str>,
	pub offset: u64,
	pub data:   Cow<'a, [u8]>,
}

impl Write<'_> {
	pub fn new<'a, H, D>(handle: H, offset: u64, data: D) -> Write<'a>
	where
		H: Into<Cow<'a, str>>,
		D: Into<Cow<'a, [u8]>>,
	{
		Write { id: 0, handle: handle.into(), offset, data: data.into() }
	}

	pub fn len(&self) -> usize {
		size_of_val(&self.id) + 4 + self.handle.len() + size_of_val(&self.offset) + 4 + self.data.len()
	}
}
