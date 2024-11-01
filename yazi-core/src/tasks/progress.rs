use serde::Serialize;
use yazi_scheduler::Ongoing;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize)]
pub struct TasksProgress {
	pub total: u32,
	pub succ:  u32,
	pub fail:  u32,

	pub found:     u64,
	pub processed: u64,
}

impl From<&Ongoing> for TasksProgress {
	fn from(ongoing: &Ongoing) -> Self {
		let mut progress = Self::default();
		if ongoing.is_empty() {
			return progress;
		}

		for task in ongoing.values() {
			progress.total += task.total;
			progress.succ += task.succ;
			progress.fail += task.fail;

			progress.found += task.found;
			progress.processed += task.processed;
		}
		progress
	}
}
