use serde::{Deserialize, Deserializer};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Opener {
	pub exec:         String,
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
			// TODO: Deprecate this field in v0.1.5
			pub cmd:  Option<String>,
			// TODO: Deprecate this field in v0.1.5
			pub args: Option<Vec<String>>,

			pub exec:         Option<String>,
			#[serde(default)]
			pub block:        bool,
			pub display_name: Option<String>,
		}

		let mut shadow = Shadow::deserialize(deserializer)?;

		// -- TODO: Deprecate this in v0.1.5
		if shadow.exec.is_none() {
			if shadow.cmd.is_none() {
				return Err(serde::de::Error::missing_field("exec"));
			}
			if shadow.args.is_none() {
				return Err(serde::de::Error::missing_field("args"));
			}

			println!(
				"WARNING: `cmd` and `args` will be deprecated in favor of `exec` in Yazi v0.1.5, see https://github.com/sxyazi/yazi/pull/45"
			);

			// Replace the $0 to $1, $1 to $2, and so on
			shadow.args = Some(
				shadow
					.args
					.unwrap()
					.into_iter()
					.map(|s| {
						if !s.starts_with('$') {
							return shell_words::quote(&s).into();
						}
						if let Ok(idx) = s[1..].parse::<usize>() {
							return format!("${}", idx + 1);
						}
						s
					})
					.collect(),
			);
			shadow.exec = Some(format!("{} {}", shadow.cmd.unwrap(), shadow.args.unwrap().join(" ")));
		}
		let exec = shadow.exec.unwrap();
		// TODO: Deprecate this in v0.1.5 --

		if exec.is_empty() {
			return Err(serde::de::Error::custom("`exec` cannot be empty"));
		}
		let display_name = if let Some(s) = shadow.display_name {
			s
		} else {
			exec.split_whitespace().next().unwrap().to_string()
		};

		let spread = exec.contains("$*") || exec.contains("$@");
		Ok(Self { exec, block: shadow.block, display_name, spread })
	}
}
