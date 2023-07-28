use anyhow::bail;
use serde::Deserialize;

#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(try_from = "String")]
pub enum PreviewAdaptor {
	Kitty,
	Ueberzug,
}

impl TryFrom<String> for PreviewAdaptor {
	type Error = anyhow::Error;

	fn try_from(value: String) -> Result<Self, Self::Error> {
		Ok(match value.to_lowercase().as_str() {
			"kitty" => Self::Kitty,
			"ueberzug" => Self::Ueberzug,
			_ => bail!("invalid preview adaptor: {}", value),
		})
	}
}
