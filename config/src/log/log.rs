use std::path::PathBuf;

use serde::{de, Deserialize, Deserializer};

use crate::MERGED_YAZI;

#[derive(Debug)]
pub struct Log {
	pub enabled: bool,
	pub root:    PathBuf,
}

impl Log {
	pub fn new() -> Self { toml::from_str(&MERGED_YAZI).unwrap() }
}

impl<'de> Deserialize<'de> for Log {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		#[derive(Deserialize)]
		struct Outer {
			log: Shadow,
		}
		#[derive(Deserialize)]
		struct Shadow {
			enabled: bool,
		}

		let outer = Outer::deserialize(deserializer)?;

		let root = xdg::BaseDirectories::with_prefix("yazi")
			.map_err(|e| de::Error::custom(e.to_string()))?
			.get_state_home();
		if !root.is_dir() {
			std::fs::create_dir_all(&root).map_err(|e| de::Error::custom(e.to_string()))?;
		}

		Ok(Self { enabled: outer.log.enabled, root })
	}
}
