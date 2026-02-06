use crate::{Task, TaskProg};

#[derive(Debug)]
pub(crate) enum PreloadOut {
	Succ,
	Fail(String),
}

impl From<mlua::Error> for PreloadOut {
	fn from(value: mlua::Error) -> Self { Self::Fail(value.to_string()) }
}

impl PreloadOut {
	pub(crate) fn reduce(self, task: &mut Task) {
		let TaskProg::Preload(prog) = &mut task.prog else { return };
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
