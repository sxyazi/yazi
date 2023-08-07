use serde::{Deserialize, Deserializer};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Opener {
	pub display_name: Option<String>,
	pub cmd:          String,
	pub args:         Vec<String>,
	pub block:        bool,
	pub spread:       bool,
}

impl<'de> Deserialize<'de> for Opener {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		#[derive(Deserialize)]
		pub struct Shadow {
			pub display_name: Option<String>,
			pub cmd:          String,
			pub args:         Vec<String>,
			#[serde(default)]
			pub block:        bool,
			#[serde(skip)]
			pub spread:       bool,
		}

		let shadow = Shadow::deserialize(deserializer)?;

		let spread = shadow.args.contains(&"$*".to_string());
		Ok(Self {
			display_name: shadow.display_name,
			cmd:          shadow.cmd,
			args:         shadow.args,
			block:        shadow.block,
			spread
		})
	}
}
