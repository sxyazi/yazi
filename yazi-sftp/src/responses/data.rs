use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Data {
	pub(crate) id:   u32,
	pub(crate) data: Vec<u8>,
}

impl Data {
	pub(crate) fn len(&self) -> usize { size_of_val(&self.id) + 4 + self.data.len() }
}
