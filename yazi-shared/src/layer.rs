use std::fmt::{self, Display};

#[derive(Debug, Default, PartialEq, Eq, Hash, Clone, Copy)]
pub enum Layer {
	#[default]
	Manager,
	Tasks,
	Select,
	Input,
	Help,
	Completion,
	Which,
}

impl Display for Layer {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::Manager => write!(f, "manager"),
			Self::Tasks => write!(f, "tasks"),
			Self::Select => write!(f, "select"),
			Self::Input => write!(f, "input"),
			Self::Help => write!(f, "help"),
			Self::Completion => write!(f, "completion"),
			Self::Which => write!(f, "which"),
		}
	}
}
