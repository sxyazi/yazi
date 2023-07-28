use serde::Deserialize;

use super::PreviewAdaptor;
use crate::config::MERGED_YAZI;

#[derive(Debug, Deserialize)]
pub struct Preview {
	pub adaptor:  PreviewAdaptor,
	pub tab_size: u32,

	pub max_width:  u32,
	pub max_height: u32,
}

impl Preview {
	pub fn new() -> Self {
		#[derive(Deserialize)]
		struct Outer {
			preview: Preview,
		}

		let outer: Outer = toml::from_str(&MERGED_YAZI).unwrap();
		outer.preview
	}
}
