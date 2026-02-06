use crate::{Task, TaskProg};

#[derive(Debug)]
pub(crate) enum FetchOutFetch {
	Succ,
	Fail(String),
}

impl From<mlua::Error> for FetchOutFetch {
	fn from(value: mlua::Error) -> Self { Self::Fail(value.to_string()) }
}

impl FetchOutFetch {
	pub(crate) fn reduce(self, task: &mut Task) {
		let TaskProg::Fetch(prog) = &mut task.prog else { return };
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
