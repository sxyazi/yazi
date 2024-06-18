use std::str::FromStr;

use serde::{Deserialize, Deserializer};

#[derive(Debug)]
pub struct Log {
	pub enabled: bool,
}

impl FromStr for Log {
	type Err = toml::de::Error;

	fn from_str(s: &str) -> Result<Self, Self::Err> { toml::from_str(s) }
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
