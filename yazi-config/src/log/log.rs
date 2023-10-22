use serde::{Deserialize, Deserializer};

use crate::MERGED_YAZI;

#[derive(Debug)]
pub struct Log {
	pub enabled: bool,
}

impl Default for Log {
	fn default() -> Self { toml::from_str(&MERGED_YAZI).unwrap() }
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

		Ok(Self { enabled: outer.log.enabled })
	}
}
