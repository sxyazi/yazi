use std::str::FromStr;

use anyhow::bail;

#[derive(Debug, Default, PartialEq, Eq, Hash, Clone, Copy)]
pub enum Layer {
	#[default]
	App,
	Manager,
	Tasks,
	Select,
	Input,
	Help,
	Completion,
	Which,
}

impl ToString for Layer {
	fn to_string(&self) -> String {
		match self {
			Self::App => "app",
			Self::Manager => "manager",
			Self::Tasks => "tasks",
			Self::Select => "select",
			Self::Input => "input",
			Self::Help => "help",
			Self::Completion => "completion",
			Self::Which => "which",
		}
		.to_string()
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
			"help" => Self::Help,
			"completion" => Self::Completion,
			"which" => Self::Which,
			_ => bail!("invalid layer: {s}"),
		})
	}
}
