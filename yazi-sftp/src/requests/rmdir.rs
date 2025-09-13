use serde::{Deserialize, Serialize};

use crate::ByteStr;

#[derive(Debug, Deserialize, Serialize)]
pub struct Rmdir<'a> {
	pub id:   u32,
	pub path: ByteStr<'a>,
}

impl<'a> Rmdir<'a> {
	pub fn new(path: impl Into<ByteStr<'a>>) -> Self { Self { id: 0, path: path.into() } }

	pub fn len(&self) -> usize { size_of_val(&self.id) + 4 + self.path.len() }
}
