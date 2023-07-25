use anyhow::bail;
use serde::Deserialize;

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Eq)]
#[serde(try_from = "String")]
pub enum SortBy {
	#[default]
	Alphabetical,
	Created,
	Modified,
	Size,
}

impl TryFrom<String> for SortBy {
	type Error = anyhow::Error;

	fn try_from(s: String) -> Result<Self, Self::Error> {
		Ok(match s.as_str() {
			"alphabetical" => Self::Alphabetical,
			"created" => Self::Created,
			"modified" => Self::Modified,
			"size" => Self::Size,
			_ => bail!("invalid sort_by value: {}", s),
		})
	}
}
