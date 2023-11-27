use anyhow::anyhow;
use yazi_shared::{Exec, Layer};

use crate::{emit, tasks::{Tasks, TasksProgress}};

pub struct Opt {
	progress: TasksProgress,
}

impl TryFrom<&Exec> for Opt {
	type Error = anyhow::Error;

	fn try_from(e: &Exec) -> Result<Self, Self::Error> {
		e.take_data().ok_or_else(|| anyhow!("invalid data"))
	}
}

impl Tasks {
	pub fn _update(progress: TasksProgress) {
		emit!(Call(Exec::call("update", vec![]).with_data(Opt { progress }).vec(), Layer::Tasks));
	}

	pub fn update(&mut self, opt: impl TryInto<Opt>) -> bool {
		let Ok(opt) = opt.try_into() else {
			return false;
		};

		self.progress = opt.progress;
		true
	}
}
