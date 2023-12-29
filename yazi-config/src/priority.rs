use std::str::FromStr;

use anyhow::anyhow;
use serde::Deserialize;

#[derive(Default, Clone, Copy, Debug, Deserialize)]
#[serde(try_from = "String")]
pub enum Priority {
	Low    = 0,
	#[default]
	Normal = 1,
	High   = 2,
}

impl FromStr for Priority {
	type Err = anyhow::Error;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s {
			"low" => Ok(Self::Low),
			"normal" => Ok(Self::Normal),
			"high" => Ok(Self::High),
			_ => Err(anyhow!("Invalid priority: {s}")),
		}
	}
}

impl TryFrom<String> for Priority {
	type Error = anyhow::Error;

	fn try_from(s: String) -> Result<Self, Self::Error> { Self::from_str(&s) }
}
