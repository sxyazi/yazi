use std::num::NonZeroUsize;

use anyhow::Result;
use lru::LruCache;
use parking_lot::Mutex;
use tokio::sync::mpsc;
use tracing::error;
use yazi_config::Priority;
use yazi_fs::FsHash64;
use yazi_runner::RUNNER;

use crate::{HIGH, LOW, TaskOp, TaskOps, fetch::{FetchIn, FetchOutFetch}};

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

	pub(crate) async fn fetch(&self, task: FetchIn) -> Result<(), FetchOutFetch> {
		let (id, fetcher) = (task.id, task.fetcher.clone());

		let hashes: Vec<_> = task.targets.iter().map(|f| f.hash_u64()).collect();
		let (state, err) = RUNNER.fetch(task.into()).await?;

		let mut loaded = self.loaded.lock();
		for (_, h) in hashes.into_iter().enumerate().filter(|&(i, _)| !state.get(i)) {
			loaded.get_mut(&h).map(|x| *x &= !(1 << fetcher.idx));
		}
		if let Some(e) = err {
			error!("Error when running fetcher '{}':\n{e}", fetcher.name);
		}

		Ok(self.ops.out(id, FetchOutFetch::Succ))
	}
}

impl Fetch {
	pub(crate) fn submit(&self, r#in: FetchIn) {
		let priority = match r#in.fetcher.prio {
			Priority::Low => LOW,
			Priority::Normal => HIGH,
			Priority::High => HIGH,
		};

		_ = self.tx.try_send(r#in, priority);
	}
}
