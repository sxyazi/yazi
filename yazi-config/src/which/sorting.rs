use std::str::FromStr;

use anyhow::bail;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(try_from = "String")]
pub enum SortBy {
	#[default]
	None,
	Key,
	Desc,
}

impl FromStr for SortBy {
	type Err = anyhow::Error;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Ok(match s {
			"none" => Self::None,
			"key" => Self::Key,
			"desc" => Self::Desc,
			_ => bail!("Invalid `sort_by` value: {s}"),
		})
	}
}

impl TryFrom<String> for SortBy {
	type Error = anyhow::Error;

	fn try_from(value: String) -> Result<Self, Self::Error> { Self::from_str(&value) }
}
