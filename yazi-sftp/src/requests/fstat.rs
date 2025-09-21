use std::borrow::Cow;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Fstat<'a> {
	pub id:     u32,
	pub handle: Cow<'a, str>,
}

impl<'a> Fstat<'a> {
	pub fn new(handle: impl Into<Cow<'a, str>>) -> Self { Self { id: 0, handle: handle.into() } }

	pub fn len(&self) -> usize { size_of_val(&self.id) + 4 + self.handle.len() }
}
