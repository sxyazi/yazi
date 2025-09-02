use crate::{Task, TaskProg};

// --- Entry
#[derive(Debug)]
pub(crate) enum PluginOutEntry {
	Succ,
	Fail(String),
}

impl From<mlua::Error> for PluginOutEntry {
	fn from(value: mlua::Error) -> Self { Self::Fail(value.to_string()) }
}

impl PluginOutEntry {
	pub(crate) fn reduce(self, task: &mut Task) {
		let TaskProg::PluginEntry(prog) = &mut task.prog else { return };
		match self {
			Self::Succ => {
				prog.state = Some(true);
			}
			Self::Fail(reason) => {
				prog.state = Some(false);
				task.log(reason);
			}
		}
	}
}
