use serde::{Deserialize, Deserializer};

use crate::config::Pattern;

#[derive(Debug, Deserialize)]
pub struct Filetype {
	pub name: Option<Pattern>,
	pub mime: Option<Pattern>,
	#[serde(default)]
	pub bg:   String,
	#[serde(default)]
	pub fg:   String,
	#[serde(default)]
	pub bold: bool,
}

impl Filetype {
	pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<Filetype>, D::Error>
	where
		D: Deserializer<'de>,
	{
		#[derive(Deserialize)]
		struct FiletypeOuter {
			rules: Vec<Filetype>,
		}

		Ok(FiletypeOuter::deserialize(deserializer)?.rules)
	}
}
