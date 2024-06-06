use std::{fmt::Display, str::FromStr};

use anyhow::bail;

#[derive(Debug, Default, PartialEq, Eq, Hash, Clone, Copy)]
pub enum Layer {
	#[default]
	App,
	Manager,
	Tasks,
	Select,
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
			Self::Select => "select",
			Self::Input => "input",
			Self::Confirm => "confirm",
			Self::Help => "help",
			Self::Completion => "completion",
			Self::Which => "which",
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
			"select" => Self::Select,
			"input" => Self::Input,
			"confirm" => Self::Confirm,
			"help" => Self::Help,
			"completion" => Self::Completion,
			"which" => Self::Which,
			_ => bail!("invalid layer: {s}"),
		})
	}
}
