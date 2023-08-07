use serde::{Deserialize, Deserializer};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Opener {
	pub cmd:          String,
	pub args:         Vec<String>,
	pub block:        bool,
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
			pub cmd:          String,
			pub args:         Vec<String>,
			#[serde(default)]
			pub block:        bool,
			pub display_name: Option<String>,
			#[serde(skip)]
			pub spread:       bool,
		}

		let shadow = Shadow::deserialize(deserializer)?;

		let display_name = if let Some(s) = shadow.display_name { s } else { shadow.cmd.clone() };
		let spread = shadow.args.contains(&"$*".to_string());

		Ok(Self { cmd: shadow.cmd, args: shadow.args, block: shadow.block, display_name, spread })
	}
}
