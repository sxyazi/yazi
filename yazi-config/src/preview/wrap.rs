use std::str::FromStr;

use anyhow::bail;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(try_from = "String")]
pub enum PreviewWrap {
	No,
	Yes,
}

impl FromStr for PreviewWrap {
	type Err = anyhow::Error;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Ok(match s {
			"no" => Self::No,
			"yes" => Self::Yes,
			_ => bail!("Invalid `wrap` value: {s}"),
		})
	}
}

impl TryFrom<String> for PreviewWrap {
	type Error = anyhow::Error;

	fn try_from(value: String) -> Result<Self, Self::Error> { Self::from_str(&value) }
}
