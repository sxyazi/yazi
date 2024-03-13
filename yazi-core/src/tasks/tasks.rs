use std::{sync::Arc, time::Duration};

use tokio::{task::JoinHandle, time::sleep};
use yazi_scheduler::{Scheduler, TaskSummary};
use yazi_shared::{emit, event::Cmd, term::Term, Layer};

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
					emit!(Call(Cmd::new("update_progress").with_data(new), Layer::App));
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
		(Term::size().rows * TASKS_PERCENT / 100).saturating_sub(TASKS_BORDER + TASKS_PADDING) as usize
	}

	pub fn paginate(&self) -> Vec<TaskSummary> {
		let ongoing = self.scheduler.ongoing.lock();
		ongoing.values().take(Self::limit()).map(Into::into).collect()
	}

	#[inline]
	pub fn len(&self) -> usize { self.scheduler.ongoing.lock().len() }
}
