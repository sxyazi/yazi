use crate::{Task, TaskProg};

// --- Fetch
#[derive(Debug)]
pub(crate) enum PreworkOutFetch {
	Succ,
	Fail(String),
}

impl From<mlua::Error> for PreworkOutFetch {
	fn from(value: mlua::Error) -> Self { Self::Fail(value.to_string()) }
}

impl PreworkOutFetch {
	pub(crate) fn reduce(self, task: &mut Task) {
		let TaskProg::PreworkFetch(prog) = &mut task.prog else { return };
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

// --- Load
#[derive(Debug)]
pub(crate) enum PreworkOutLoad {
	Succ,
	Fail(String),
}

impl From<mlua::Error> for PreworkOutLoad {
	fn from(value: mlua::Error) -> Self { Self::Fail(value.to_string()) }
}

impl PreworkOutLoad {
	pub(crate) fn reduce(self, task: &mut Task) {
		let TaskProg::PreworkLoad(prog) = &mut task.prog else { return };
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

// --- Size
#[derive(Debug)]
pub(crate) enum PreworkOutSize {
	Done,
}

impl PreworkOutSize {
	pub(crate) fn reduce(self, task: &mut Task) {
		let TaskProg::PreworkSize(prog) = &mut task.prog else { return };
		match self {
			Self::Done => {
				prog.done = true;
			}
		}
	}
}
