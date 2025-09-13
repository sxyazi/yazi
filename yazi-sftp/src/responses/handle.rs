use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Handle {
	pub id:     u32,
	pub handle: String,
}

impl Handle {
	pub fn len(&self) -> usize { size_of_val(&self.id) + 4 + self.handle.len() }
}
