use hashbrown::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Init {
	version:    u32,
	extensions: HashMap<String, String>,
}

impl Init {
	fn new(extensions: HashMap<String, String>) -> Self { Self { version: 3, extensions } }

	pub(crate) fn len(&self) -> usize {
		size_of_val(&self.version)
			+ self.extensions.iter().map(|(k, v)| 4 + k.len() + 4 + v.len()).sum::<usize>()
	}
}

impl Default for Init {
	fn default() -> Self { Self::new(HashMap::new()) }
}
