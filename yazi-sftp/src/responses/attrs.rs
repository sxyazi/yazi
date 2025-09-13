use serde::{Deserialize, Serialize};

use crate::fs;

#[derive(Debug, Deserialize, Serialize)]
pub struct Attrs {
	pub id:    u32,
	pub attrs: fs::Attrs,
}

impl Attrs {
	pub fn len(&self) -> usize { size_of_val(&self.id) + self.attrs.len() }
}
