use std::{fmt::Display, str::FromStr};

use anyhow::bail;

#[derive(Debug, Default, PartialEq, Eq, Hash, Clone, Copy)]
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
	Mount,
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
			Self::Mount => "mount",
		})
	}
}

impl FromStr for Layer {
	type Err = anyhow::Error;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Ok(match s {
			"app" => Self::App,
			"manager" => Self::Manager,
			"tasks" => Self::Tasks,
			"spot" => Self::Spot,
			"pick" => Self::Pick,
			"input" => Self::Input,
			"confirm" => Self::Confirm,
			"help" => Self::Help,
			"completion" => Self::Completion,
			"which" => Self::Which,
			"mount" => Self::Mount,
			_ => bail!("invalid layer: {s}"),
		})
	}
}
