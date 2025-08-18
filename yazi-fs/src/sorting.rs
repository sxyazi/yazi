use std::{fmt::Display, str::FromStr};

use serde::{Deserialize, Serialize};

// --- by
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum SortBy {
	#[default]
	None,
	Mtime,
	Btime,
	Extension,
	Alphabetical,
	Natural,
	Size,
	Random,
}

impl FromStr for SortBy {
	type Err = serde::de::value::Error;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Self::deserialize(serde::de::value::StrDeserializer::new(s))
	}
}

impl Display for SortBy {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.write_str(match self {
			Self::None => "none",
			Self::Mtime => "mtime",
			Self::Btime => "btime",
			Self::Extension => "extension",
			Self::Alphabetical => "alphabetical",
			Self::Natural => "natural",
			Self::Size => "size",
			Self::Random => "random",
		})
	}
}

// --- fallback
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum SortFallback {
	#[default]
	Alphabetical,
	Natural,
}

impl FromStr for SortFallback {
	type Err = serde::de::value::Error;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Self::deserialize(serde::de::value::StrDeserializer::new(s))
	}
}

impl Display for SortFallback {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.write_str(match self {
			Self::Alphabetical => "alphabetical",
			Self::Natural => "natural",
		})
	}
}
