use std::num::NonZeroUsize;

use anyhow::Result;
use lru::LruCache;
use parking_lot::Mutex;
use tokio::sync::mpsc;
use yazi_config::Priority;
use yazi_macro::error;
use yazi_runner::RUNNER;

use crate::{HIGH, LOW, TaskOp, TaskOps, fetch::{FetchIn, FetchInFetch, FetchOutFetch}};

pub struct Fetch {
	ops:        TaskOps,
	tx:         async_priority_channel::Sender<FetchIn, u8>,
	pub loaded: Mutex<LruCache<u64, u16>>,
}

impl Fetch {
	pub(crate) fn new(
		ops: &mpsc::UnboundedSender<TaskOp>,
		tx: async_priority_channel::Sender<FetchIn, u8>,
	) -> Self {
		Self {
			ops: ops.into(),
			tx,
			loaded: Mutex::new(LruCache::new(NonZeroUsize::new(4096).unwrap())),
		}
	}

	pub(crate) async fn fetch(&self, task: FetchInFetch) -> Result<(), FetchOutFetch> {
		let (id, fetcher) = (task.id, task.fetcher.clone());

		for status in RUNNER.fetch(task.into()).await? {
			if status.retry {
				self.loaded.lock().get_mut(&status.hash).map(|x| *x &= !(1 << fetcher.idx));
			}
			if let Some(e) = status.error {
				error!("Error when running fetcher '{}':\n{e:?}", fetcher.name);
			}
		}

		Ok(self.ops.out(id, FetchOutFetch::Succ))
	}
}

impl Fetch {
	pub(crate) fn submit(&self, r#in: impl Into<FetchIn>) {
		let r#in = r#in.into();
		let priority = match &r#in {
			FetchIn::Fetch(r#in) => match r#in.fetcher.prio {
				Priority::Low => LOW,
				Priority::Normal => HIGH,
				Priority::High => HIGH,
			},
			FetchIn::Custom(_) => LOW,
		};

		_ = self.tx.try_send(r#in, priority);
	}
}
