use anyhow::Result;
use serde::{Deserialize, Deserializer};
use std::collections::HashMap;

use crate::MERGED_YAZI;

/// Schemes in configuration file.
#[derive(Debug)]
pub struct Schemes {
	pub rules: Vec<Scheme>,
}

impl Default for Schemes {
	fn default() -> Self {
		toml::from_str(&MERGED_YAZI).unwrap()
	}
}

impl Schemes {
	/// Consume loaded config to build yazi_shared::fs::Schemes.
	pub fn make(self) -> Result<yazi_shared::fs::Schemes> {
		yazi_shared::fs::Schemes::from_iter(self.rules.into_iter().map(|s| (s.name, s.typ, s.config)))
	}
}

impl<'de> Deserialize<'de> for Schemes {
	fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		#[derive(Deserialize, Default)]
		#[serde(default)]
		struct Outer {
			schemes: Shadow,
		}
		#[derive(Deserialize, Default)]
		struct Shadow {
			rules: Vec<Scheme>,
		}

		let outer = Outer::deserialize(deserializer)?;

		Ok(Self { rules: outer.schemes.rules })
	}
}

/// Scheme in configuration file.
#[derive(Debug, Deserialize)]
pub struct Scheme {
	pub name: String,
	#[serde(rename = "type")]
	pub typ: String,
	pub config: HashMap<String, String>,
}
