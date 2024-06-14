use std::collections::HashMap;

use anyhow::Result;
use serde::{Deserialize, Deserializer};

/// Schemes in configuration file.
#[derive(Debug)]
pub struct Schemes {
	pub rules: Vec<Scheme>,
}

impl Default for Schemes {
	fn default() -> Self {
		let schemes_path = yazi_shared::Xdg::state_dir().join("schemes.toml");
		// Ignore any errors, as it's fine to have an empty file.
		let schemes_content = std::fs::read_to_string(&schemes_path).unwrap_or_default();
		toml::from_str(&schemes_content).unwrap()
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
