use serde::Deserialize;

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(rename_all = "kebab-case")]
pub enum Platform {
	#[default]
	All,
	Linux,
	Macos,
	Windows,
	Android,
	Unix,
}

impl Platform {
	pub(crate) fn matches(self) -> bool {
		match self {
			Self::All => true,
			Self::Linux => cfg!(target_os = "linux"),
			Self::Macos => cfg!(target_os = "macos"),
			Self::Windows => cfg!(windows),
			Self::Android => cfg!(target_os = "android"),
			Self::Unix => cfg!(unix),
		}
	}
}
