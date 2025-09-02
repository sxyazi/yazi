use crate::{Task, TaskProg};

#[derive(Debug)]
pub(crate) enum ProcessOutBlock {
	Succ,
	Fail(String),
}

impl From<std::io::Error> for ProcessOutBlock {
	fn from(value: std::io::Error) -> Self { Self::Fail(value.to_string()) }
}

impl ProcessOutBlock {
	pub(crate) fn reduce(self, task: &mut Task) {
		let TaskProg::ProcessBlock(prog) = &mut task.prog else { return };
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

// --- Orphan
#[derive(Debug)]
pub(crate) enum ProcessOutOrphan {
	Succ,
	Fail(String),
}

impl From<anyhow::Error> for ProcessOutOrphan {
	fn from(value: anyhow::Error) -> Self { Self::Fail(value.to_string()) }
}

impl ProcessOutOrphan {
	pub(crate) fn reduce(self, task: &mut Task) {
		let TaskProg::ProcessOrphan(prog) = &mut task.prog else { return };
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

// --- Bg
#[derive(Debug)]
pub(crate) enum ProcessOutBg {
	Log(String),
	Succ,
	Fail(String),
}

impl From<anyhow::Error> for ProcessOutBg {
	fn from(value: anyhow::Error) -> Self { Self::Fail(value.to_string()) }
}

impl ProcessOutBg {
	pub(crate) fn reduce(self, task: &mut Task) {
		let TaskProg::ProcessBg(prog) = &mut task.prog else { return };
		match self {
			Self::Log(line) => {
				task.log(line);
			}
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
