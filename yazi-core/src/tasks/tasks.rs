use std::{sync::Arc, time::Duration};

use tokio::{task::JoinHandle, time::sleep};
use yazi_emulator::Dimension;
use yazi_scheduler::{Scheduler, TaskSnap, TaskSummary};

use super::{TASKS_BORDER, TASKS_PADDING, TASKS_PERCENT};
use crate::AppProxy;

pub struct Tasks {
	pub scheduler: Arc<Scheduler>,
	handle:        JoinHandle<()>,

	pub visible: bool,
	pub cursor:  usize,
	pub snaps:   Vec<TaskSnap>,
	pub summary: TaskSummary,
}

impl Tasks {
	pub fn serve() -> Self {
		let scheduler = Scheduler::serve();
		let ongoing = scheduler.ongoing.clone();

		let handle = tokio::spawn(async move {
			let mut last = TaskSummary::default();
			loop {
				sleep(Duration::from_millis(500)).await;

				let new = TaskSummary::from(&*ongoing.lock());
				if last != new {
					last = new;
					AppProxy::update_progress(new);
				}
			}
		});

		Self {
			scheduler: Arc::new(scheduler),
			handle,

			visible: false,
			cursor: 0,
			snaps: Default::default(),
			summary: Default::default(),
		}
	}

	pub fn shutdown(&self) {
		self.scheduler.shutdown();
		self.handle.abort();
	}

	pub fn limit() -> usize {
		((Dimension::available().rows * TASKS_PERCENT / 100)
			.saturating_sub(TASKS_BORDER + TASKS_PADDING) as usize)
			/ 3
	}

	pub fn paginate(&self) -> Vec<TaskSnap> {
		self.scheduler.ongoing.lock().values().take(Self::limit()).map(Into::into).collect()
	}
}
