use serde::{Deserialize, Serialize};

use crate::ByteStr;

#[derive(Debug, Deserialize, Serialize)]
pub struct Lstat<'a> {
	pub id:   u32,
	pub path: ByteStr<'a>,
}

impl Lstat<'_> {
	pub fn new<'a>(path: impl Into<ByteStr<'a>>) -> Lstat<'a> { Lstat { id: 0, path: path.into() } }

	pub fn len(&self) -> usize { size_of_val(&self.id) + 4 + self.path.len() }
}
