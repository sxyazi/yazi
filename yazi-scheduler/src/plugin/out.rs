use yazi_runner::entry::EntryError;

use crate::{Task, TaskProg};

// --- Entry
#[derive(Debug)]
pub(crate) enum PluginOutEntry {
	Succ,
	Fail(String),
	Cancel,
}

impl From<mlua::Error> for PluginOutEntry {
	fn from(value: mlua::Error) -> Self {
		if value.downcast_ref().is_some_and(EntryError::is_cancelled) {
			Self::Cancel
		} else {
			Self::Fail(value.to_string())
		}
	}
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
			Self::Cancel => task.cancel(),
		}
	}
}
