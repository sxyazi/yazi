use anyhow::bail;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(try_from = "String")]
pub enum Linemode {
	#[default]
	None,
	Size,
	Mtime,
	Permissions,
}

impl TryFrom<String> for Linemode {
	type Error = anyhow::Error;

	fn try_from(s: String) -> Result<Self, Self::Error> {
		Ok(match s.as_str() {
			"none" => Self::None,
			"size" => Self::Size,
			"mtime" => Self::Mtime,
			"permissions" => Self::Permissions,
			_ => bail!("invalid linemode value: {s}"),
		})
	}
}
