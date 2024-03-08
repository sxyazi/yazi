use serde::{Deserialize, Deserializer};

use crate::MERGED_YAZI;

#[derive(Debug)]
pub struct Headsup {
	// TODO: remove this once Yazi 0.3 is released --
	pub disable_exec_warn: bool,
}

impl Default for Headsup {
	fn default() -> Self { toml::from_str(&MERGED_YAZI).unwrap() }
}

impl<'de> Deserialize<'de> for Headsup {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		#[derive(Deserialize)]
		struct Outer {
			headsup: Shadow,
		}
		#[derive(Deserialize)]
		struct Shadow {
			#[serde(default)]
			disable_exec_warn: bool,
		}

		let outer = Outer::deserialize(deserializer)?;

		Ok(Self { disable_exec_warn: outer.headsup.disable_exec_warn })
	}
}
