use std::str::FromStr;

use anyhow::bail;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(try_from = "String")]
pub enum SortBy {
	#[default]
	None,
	Modified,
	Created,
	Extension,
	Alphabetical,
	Natural,
	Size,
}

impl FromStr for SortBy {
	type Err = anyhow::Error;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Ok(match s {
			"none" => Self::None,
			"modified" => Self::Modified,
			"created" => Self::Created,
			"extension" => Self::Extension,
			"alphabetical" => Self::Alphabetical,
			"natural" => Self::Natural,
			"size" => Self::Size,
			_ => bail!("invalid sort_by value: {s}"),
		})
	}
}

impl TryFrom<String> for SortBy {
	type Error = anyhow::Error;

	fn try_from(s: String) -> Result<Self, Self::Error> { Self::from_str(&s) }
}

impl ToString for SortBy {
	fn to_string(&self) -> String {
		match self {
			Self::None => "none",
			Self::Modified => "modified",
			Self::Created => "created",
			Self::Extension => "extension",
			Self::Alphabetical => "alphabetical",
			Self::Natural => "natural",
			Self::Size => "size",
		}
		.to_string()
	}
}
