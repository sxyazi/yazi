use std::borrow::Cow;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct ReadDir<'a> {
	pub(crate) id: u32,
	handle:        Cow<'a, str>,
}

impl<'a> ReadDir<'a> {
	pub(crate) fn new(handle: &'a str) -> Self { Self { id: 0, handle: handle.into() } }

	pub(crate) fn len(&self) -> usize { size_of_val(&self.id) + 4 + self.handle.len() }
}
