use yazi_shared::{emit, event::Exec, render, Layer};

use crate::tasks::{Tasks, TasksProgress};

pub struct Opt {
	progress: TasksProgress,
}

impl TryFrom<&Exec> for Opt {
	type Error = ();

	fn try_from(e: &Exec) -> Result<Self, Self::Error> { e.take_data().ok_or(()) }
}

impl Tasks {
	pub fn _update(progress: TasksProgress) {
		emit!(Call(Exec::call("update", vec![]).with_data(Opt { progress }).vec(), Layer::Tasks));
	}

	pub fn update(&mut self, opt: impl TryInto<Opt>) {
		let Ok(opt) = opt.try_into() else {
			return;
		};

		self.progress = opt.progress;
		render!();
	}
}
