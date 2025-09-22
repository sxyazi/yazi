use std::{fmt::Display, str::FromStr};

use serde::{Deserialize, Deserializer, Serialize};

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
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

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum SortByMulti {
	Single(SortBy),
	Multiple(Vec<SortBy>),
}

impl Default for SortByMulti {
	fn default() -> Self {
		Self::Single(SortBy::default())
	}
}

impl SortByMulti {
	pub fn methods(&self) -> &[SortBy] {
		match self {
			Self::Single(sort_by) => std::slice::from_ref(sort_by),
			Self::Multiple(sort_methods) => sort_methods,
		}
	}

	pub fn primary(&self) -> SortBy {
		match self {
			Self::Single(sort_by) => *sort_by,
			Self::Multiple(sort_methods) => sort_methods.first().copied().unwrap_or_default(),
		}
	}
}

impl<'de> Deserialize<'de> for SortByMulti {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		#[derive(Deserialize)]
		#[serde(untagged)]
		enum Helper {
			Single(SortBy),
			Multiple(Vec<SortBy>),
		}

		match Helper::deserialize(deserializer)? {
			Helper::Single(sort_by) => Ok(Self::Single(sort_by)),
			Helper::Multiple(methods) => {
				if methods.is_empty() {
					Ok(Self::Single(SortBy::None))
				} else {
					Ok(Self::Multiple(methods))
				}
			}
		}
	}
}

impl From<SortBy> for SortByMulti {
	fn from(sort_by: SortBy) -> Self {
		Self::Single(sort_by)
	}
}

impl Display for SortByMulti {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Single(sort_by) => sort_by.fmt(f),
			Self::Multiple(methods) => {
				let methods_str: Vec<String> = methods.iter().map(|m| m.to_string()).collect();
				write!(f, "[{}]", methods_str.join(", "))
			}
		}
	}
}
