use std::{sync::Arc, time::Duration};

use parking_lot::Mutex;
use tokio::{task::JoinHandle, time::sleep};
use yazi_adapter::Dimension;
use yazi_parser::app::TaskSummary;
use yazi_proxy::AppProxy;
use yazi_scheduler::{Ongoing, Scheduler, TaskSnap};

use super::{TASKS_BORDER, TASKS_PADDING, TASKS_PERCENT};

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

				let new = ongoing.lock().summary();
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
		self.ongoing().lock().values().take(Self::limit()).map(Into::into).collect()
	}

	pub fn ongoing(&self) -> &Arc<Mutex<Ongoing>> { &self.scheduler.ongoing }
}
