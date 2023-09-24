use serde::{Deserialize, Deserializer};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Opener {
	pub exec:         String,
	pub block:        bool,
	pub orphan:       bool,
	pub display_name: String,
	pub spread:       bool,
}

impl<'de> Deserialize<'de> for Opener {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		#[derive(Deserialize)]
		pub struct Shadow {
			pub exec:         String,
			#[serde(default)]
			pub block:        bool,
			#[serde(default)]
			pub orphan:       bool,
			pub display_name: Option<String>,
		}

		let shadow = Shadow::deserialize(deserializer)?;

		if shadow.exec.is_empty() {
			return Err(serde::de::Error::custom("`exec` cannot be empty"));
		}

		let display_name = shadow
			.display_name
			.unwrap_or_else(|| shadow.exec.split_whitespace().next().unwrap().to_string());

		let spread = shadow.exec.contains("$*") || shadow.exec.contains("$@");
		Ok(Self { exec: shadow.exec, block: shadow.block, orphan: shadow.orphan, display_name, spread })
	}
}
