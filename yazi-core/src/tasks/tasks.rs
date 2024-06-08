use std::{sync::Arc, time::Duration};

use parking_lot::Mutex;
use tokio::{task::JoinHandle, time::sleep};
use yazi_adapter::Dimension;
use yazi_scheduler::{Ongoing, Scheduler, TaskSummary};
use yazi_shared::{emit, event::Cmd, Layer};

use super::{TasksProgress, TASKS_BORDER, TASKS_PADDING, TASKS_PERCENT};

pub struct Tasks {
	pub(super) scheduler: Arc<Scheduler>,
	handle:               JoinHandle<()>,

	pub visible:   bool,
	pub cursor:    usize,
	pub progress:  TasksProgress,
	pub summaries: Vec<TaskSummary>,
}

impl Tasks {
	pub fn serve() -> Self {
		let scheduler = Scheduler::serve();
		let ongoing = scheduler.ongoing.clone();

		let handle = tokio::spawn(async move {
			let mut last = TasksProgress::default();
			loop {
				sleep(Duration::from_millis(500)).await;

				let new = TasksProgress::from(&*ongoing.lock());
				if last != new {
					last = new;
					emit!(Call(Cmd::new("update_progress").with_any("progress", new), Layer::App));
				}
			}
		});

		Self {
			scheduler: Arc::new(scheduler),
			handle,

			visible: false,
			cursor: 0,
			progress: Default::default(),
			summaries: Default::default(),
		}
	}

	pub fn shutdown(&self) {
		self.scheduler.shutdown();
		self.handle.abort();
	}

	#[inline]
	pub fn limit() -> usize {
		(Dimension::available().rows * TASKS_PERCENT / 100).saturating_sub(TASKS_BORDER + TASKS_PADDING)
			as usize
	}

	pub fn paginate(&self) -> Vec<TaskSummary> {
		self.ongoing().lock().values().take(Self::limit()).map(Into::into).collect()
	}

	#[inline]
	pub fn ongoing(&self) -> &Arc<Mutex<Ongoing>> { &self.scheduler.ongoing }
}
