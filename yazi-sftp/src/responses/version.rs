use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Version {
	pub version:    u32,
	pub extensions: HashMap<String, String>,
}

impl Version {
	pub fn len(&self) -> usize {
		size_of_val(&self.version)
			+ self.extensions.iter().map(|(k, v)| 4 + k.len() + 4 + v.len()).sum::<usize>()
	}
}
