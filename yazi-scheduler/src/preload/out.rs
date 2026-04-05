use yazi_runner::preloader::PreloadError;

use crate::{CleanupState, Task, TaskProg};

#[derive(Debug)]
pub(crate) enum PreloadOut {
	Succ,
	Fail(String),
	Clean,
}

impl From<PreloadError> for PreloadOut {
	fn from(value: PreloadError) -> Self { Self::Fail(value.to_string()) }
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
			Self::Clean => {
				prog.cleaned = CleanupState::Success;
			}
		}
	}
}
