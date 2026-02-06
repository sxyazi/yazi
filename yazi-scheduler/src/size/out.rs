use crate::{Task, TaskProg};

#[derive(Debug)]
pub(crate) enum SizeOut {
	Done,
}

impl SizeOut {
	pub(crate) fn reduce(self, task: &mut Task) {
		let TaskProg::Size(prog) = &mut task.prog else { return };
		match self {
			Self::Done => {
				prog.done = true;
			}
		}
	}
}
