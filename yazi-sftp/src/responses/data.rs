use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Data {
	pub id:   u32,
	pub data: Vec<u8>,
}

impl Data {
	pub fn len(&self) -> usize { size_of_val(&self.id) + 4 + self.data.len() }
}
