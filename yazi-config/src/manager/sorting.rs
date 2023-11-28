use std::str::FromStr;

use anyhow::bail;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(try_from = "String")]
pub enum SortBy {
	#[default]
	None,
	Alphabetical,
	Created,
	Modified,
	Natural,
	Size,
	Extension,
}

impl FromStr for SortBy {
	type Err = anyhow::Error;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Ok(match s {
			"none" => Self::None,
			"alphabetical" => Self::Alphabetical,
			"created" => Self::Created,
			"modified" => Self::Modified,
			"natural" => Self::Natural,
			"size" => Self::Size,
			"extension" => Self::Extension,
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
			Self::Alphabetical => "alphabetical",
			Self::Created => "created",
			Self::Modified => "modified",
			Self::Natural => "natural",
			Self::Size => "size",
			Self::Extension => "extension",
		}
		.to_string()
	}
}
