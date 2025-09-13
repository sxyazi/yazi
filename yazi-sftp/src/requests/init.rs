use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Init {
	pub version:    u32,
	pub extensions: HashMap<String, String>,
}

impl Init {
	pub fn new(extensions: HashMap<String, String>) -> Self { Self { version: 3, extensions } }

	pub fn len(&self) -> usize {
		size_of_val(&self.version)
			+ self.extensions.iter().map(|(k, v)| 4 + k.len() + 4 + v.len()).sum::<usize>()
	}
}

impl Default for Init {
	fn default() -> Self { Self::new(HashMap::new()) }
}
