use std::borrow::Cow;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct ReadDir<'a> {
	pub id:     u32,
	pub handle: Cow<'a, str>,
}

impl<'a> ReadDir<'a> {
	pub fn new(handle: &'a str) -> Self { Self { id: 0, handle: handle.into() } }

	pub fn len(&self) -> usize { size_of_val(&self.id) + 4 + self.handle.len() }
}
