use serde::Serialize;
use yazi_scheduler::Running;

#[derive(Clone, Copy, Default, Eq, PartialEq, Serialize)]
pub struct TasksProgress {
	pub total: u32,
	pub succ:  u32,
	pub fail:  u32,

	pub found:     u64,
	pub processed: u64,
}

impl From<&Running> for TasksProgress {
	fn from(running: &Running) -> Self {
		let mut progress = Self::default();
		if running.is_empty() {
			return progress;
		}

		for task in running.values() {
			progress.total += task.total;
			progress.succ += task.succ;
			progress.fail += task.fail;

			progress.found += task.found;
			progress.processed += task.processed;
		}
		progress
	}
}
