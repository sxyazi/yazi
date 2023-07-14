use anyhow::bail;
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone, Copy)]
#[serde(try_from = "String")]
pub enum SortBy {
	Alphabetical,
	Created,
	Modified,
	Size,
}

impl TryFrom<String> for SortBy {
	type Error = anyhow::Error;

	fn try_from(s: String) -> Result<Self, Self::Error> {
		Ok(match s.as_str() {
			"created" => Self::Created,
			"modified" => Self::Modified,
			"size" => Self::Size,
			"alphabetical" => Self::Alphabetical,
			_ => bail!("invalid sort_by value: {}", s),
		})
	}
}
