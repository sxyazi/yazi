use std::path::Path;

use anyhow::Context;
use serde::{Deserialize, Serialize};
use yazi_shared::RoCell;

use crate::{Preset, MERGED_THEME};

#[derive(Deserialize, Serialize)]
pub struct Flavor {
	#[serde(rename = "use")]
	pub use_: String,
}

impl Default for Flavor {
	fn default() -> Self {
		#[derive(Deserialize)]
		struct Outer {
			flavor: Flavor,
		}

		toml::from_str::<Outer>(&MERGED_THEME).unwrap().flavor
	}
}

impl Flavor {
	pub fn merge_with(&self, merged: &RoCell<String>, p: &Path) {
		if self.use_.is_empty() {
			return;
		}

		let path = p.join(format!("flavors/{}.yazi/theme.toml", self.use_));
		let s = std::fs::read_to_string(&path)
			.with_context(|| format!("Failed to load flavor from: {:?}", path))
			.unwrap();

		merged.replace(Preset::merge_str(&s, merged));
	}
}
