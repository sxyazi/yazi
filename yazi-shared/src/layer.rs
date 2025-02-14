use std::{fmt::Display, str::FromStr};

use serde::Deserialize;

#[derive(Debug, Default, PartialEq, Eq, Hash, Clone, Copy, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Layer {
	#[default]
	App,
	Manager,
	Tasks,
	Spot,
	Pick,
	Input,
	Confirm,
	Help,
	Completion,
	Which,
}

impl Display for Layer {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.write_str(match self {
			Self::App => "app",
			Self::Manager => "manager",
			Self::Tasks => "tasks",
			Self::Spot => "spot",
			Self::Pick => "pick",
			Self::Input => "input",
			Self::Confirm => "confirm",
			Self::Help => "help",
			Self::Completion => "completion",
			Self::Which => "which",
		})
	}
}

impl FromStr for Layer {
	type Err = serde::de::value::Error;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Self::deserialize(serde::de::value::StrDeserializer::new(s))
	}
}
