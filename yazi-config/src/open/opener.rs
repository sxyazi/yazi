use serde::{Deserialize, Deserializer};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Opener {
	pub run:    String,
	pub block:  bool,
	pub orphan: bool,
	pub desc:   String,
	pub for_:   Option<String>,
	pub spread: bool,
}

impl Opener {
	pub fn take(mut self) -> Option<Self> {
		if let Some(for_) = self.for_.take() {
			match for_.as_bytes() {
				b"unix" if cfg!(unix) => {}
				b"windows" if cfg!(windows) => {}
				b"linux" if cfg!(target_os = "linux") => {}
				b"macos" if cfg!(target_os = "macos") => {}
				_ => return None,
			}
		}
		Some(self)
	}
}

impl<'de> Deserialize<'de> for Opener {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		#[derive(Deserialize)]
		pub struct Shadow {
			run:    String,
			#[serde(default)]
			block:  bool,
			#[serde(default)]
			orphan: bool,
			desc:   Option<String>,
			#[serde(rename = "for")]
			for_:   Option<String>,
		}

		let shadow = Shadow::deserialize(deserializer)?;

		let run = shadow.run;
		if run.is_empty() {
			return Err(serde::de::Error::custom("`run` cannot be empty"));
		}

		let desc = shadow.desc.unwrap_or_else(|| run.split_whitespace().next().unwrap().to_string());

		let spread = run.contains("$@") || run.contains("%*") || run.contains("$*");
		Ok(Self { run, block: shadow.block, orphan: shadow.orphan, desc, for_: shadow.for_, spread })
	}
}
