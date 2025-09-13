use serde::{Deserialize, Serialize};

use crate::ByteStr;

#[derive(Debug, Deserialize, Serialize)]
pub struct Rename<'a> {
	pub id:   u32,
	pub from: ByteStr<'a>,
	pub to:   ByteStr<'a>,
}

impl<'a> Rename<'a> {
	pub fn new<F, T>(from: F, to: T) -> Self
	where
		F: Into<ByteStr<'a>>,
		T: Into<ByteStr<'a>>,
	{
		Self { id: 0, from: from.into(), to: to.into() }
	}

	pub fn len(&self) -> usize { size_of_val(&self.id) + 4 + self.from.len() + 4 + self.to.len() }
}
