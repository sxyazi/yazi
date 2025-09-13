use serde::{Deserialize, Serialize};

use crate::{ByteStr, fs::Attrs};

#[derive(Debug, Deserialize, Serialize)]
pub struct Mkdir<'a> {
	pub id:    u32,
	pub path:  ByteStr<'a>,
	pub attrs: Attrs,
}

impl<'a> Mkdir<'a> {
	pub fn new<P>(path: P, attrs: Attrs) -> Self
	where
		P: Into<ByteStr<'a>>,
	{
		Self { id: 0, path: path.into(), attrs }
	}

	pub fn len(&self) -> usize { size_of_val(&self.id) + 4 + self.path.len() + self.attrs.len() }
}
