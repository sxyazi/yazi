use hashbrown::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Version {
	version:               u32,
	pub(crate) extensions: HashMap<String, String>,
}

impl Version {
	pub(crate) fn len(&self) -> usize {
		size_of_val(&self.version)
			+ self.extensions.iter().map(|(k, v)| 4 + k.len() + 4 + v.len()).sum::<usize>()
	}
}
