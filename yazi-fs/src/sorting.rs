use std::{fmt::Display, str::FromStr};

use serde::{Deserialize, Serialize, Serializer};

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

#[derive(Clone, Debug, PartialEq, Default)]
pub struct SortBys(pub Vec<SortBy>);

#[derive(Deserialize)]
#[serde(untagged)]
enum SortBysDef {
	Single(SortBy),
	Multiple(Vec<SortBy>),
}

impl<'de> Deserialize<'de> for SortBys {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		let helper = SortBysDef::deserialize(deserializer)?;
		Ok(match helper {
			SortBysDef::Single(s) => SortBys(vec![s]),
			SortBysDef::Multiple(v) => SortBys(v),
		})
	}
}

impl Serialize for SortBys {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: Serializer,
	{
		match self.0.len() {
			1 => {
				// serialize the single SortBy as a single value
				self.0[0].serialize(serializer)
			}
			_ => {
				// empty or multi -> serialize as an array
				self.0.serialize(serializer)
			}
		}
	}
}

impl Display for SortBys {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let mut iter = self.0.iter();
		f.write_str("[")?;
		if let Some(first) = iter.next() {
			f.write_str(&first.to_string())?;
			for sort_by in iter {
				f.write_str(", ")?;
				f.write_str(&sort_by.to_string())?;
			}
		}
		f.write_str("]")?;
		Ok(())
	}
}
