use serde::{Deserialize, Serialize};

use crate::fs;

#[derive(Debug, Deserialize, Serialize)]
pub struct Attrs {
	pub(crate) id:    u32,
	pub(crate) attrs: fs::Attrs,
}

impl Attrs {
	pub(crate) fn len(&self) -> usize { size_of_val(&self.id) + self.attrs.len() }
}
