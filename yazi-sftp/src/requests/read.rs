use std::borrow::Cow;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Read<'a> {
	pub id:     u32,
	pub handle: Cow<'a, str>,
	pub offset: u64,
	pub len:    u32,
}

impl<'a> Read<'a> {
	pub fn new<H>(handle: H, offset: u64, len: u32) -> Self
	where
		H: Into<Cow<'a, str>>,
	{
		Self { id: 0, handle: handle.into(), offset, len }
	}

	pub fn len(&self) -> usize {
		size_of_val(&self.id)
			+ 4 + self.handle.len()
			+ size_of_val(&self.offset)
			+ size_of_val(&self.len)
	}
}
