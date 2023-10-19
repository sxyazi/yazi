use serde::{Deserialize, Deserializer};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Opener {
	pub exec:   String,
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
			exec:   String,
			#[serde(default)]
			block:  bool,
			#[serde(default)]
			orphan: bool,
			desc:   Option<String>,
			#[serde(rename = "for")]
			for_:   Option<String>,

			// TODO: remove this when v1.0.5 is released --
			display_name: Option<String>,
			// TODO: -- remove this when v1.0.5 is released
		}

		let mut shadow = Shadow::deserialize(deserializer)?;
		if shadow.exec.is_empty() {
			return Err(serde::de::Error::custom("`exec` cannot be empty"));
		}

		// TODO: remove this when v1.0.5 is released --
		if shadow.display_name.is_some() {
			println!(
				"WARNING: `display_name` is deprecated and will be removed in Yazi v1.0.6. Use `desc` instead.\ne.g. {}\n\n",
				r#"{ exec = 'nvim "$@"', display_name = "Edit" }  ==>  { exec = 'nvim "$@"', desc = "Edit" }"#
			);
		}
		if shadow.display_name.is_some() && shadow.desc.is_none() {
			shadow.desc = shadow.display_name.clone();
		}
		// TODO: -- remove this when v1.0.5 is released

		let desc =
			shadow.desc.unwrap_or_else(|| shadow.exec.split_whitespace().next().unwrap().to_string());

		let spread =
			shadow.exec.contains("$@") || shadow.exec.contains("%*") || shadow.exec.contains("$*");
		Ok(Self {
			exec: shadow.exec,
			block: shadow.block,
			orphan: shadow.orphan,
			desc,
			for_: shadow.for_,
			spread,
		})
	}
}
