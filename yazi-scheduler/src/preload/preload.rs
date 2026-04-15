use std::num::NonZeroUsize;

use anyhow::Result;
use lru::LruCache;
use parking_lot::Mutex;
use tokio::sync::mpsc;
use tracing::error;
use yazi_config::Priority;
use yazi_fs::FsHash64;
use yazi_runner::{RUNNER, preloader::{PreloadError, PreloadJob}};

use crate::{HIGH, LOW, NORMAL, TaskOp, TaskOps, preload::{PreloadIn, PreloadOut}};

pub struct Preload {
	ops: TaskOps,
	tx:  async_priority_channel::Sender<PreloadIn, u8>,

	pub loaded:  Mutex<LruCache<u64, u16>>,
	pub loading: Mutex<LruCache<u64, yazi_shared::Id>>,
}

impl Preload {
	pub(crate) fn new(
		ops: &mpsc::UnboundedSender<TaskOp>,
		tx: async_priority_channel::Sender<PreloadIn, u8>,
	) -> Self {
		Self {
			ops: ops.into(),
			tx,

			loaded: Mutex::new(LruCache::new(NonZeroUsize::new(4096).unwrap())),
			loading: Mutex::new(LruCache::new(NonZeroUsize::new(256).unwrap())),
		}
	}

	pub(crate) async fn preload(&self, task: PreloadIn) -> Result<(), PreloadOut> {
		let hash = task.target.hash_u64();

		let mut rx =
			RUNNER.preload(PreloadJob { preloader: task.preloader.clone(), file: task.target }).await;
		let state = match rx.recv().await.unwrap_or(Err(PreloadError::Cancelled)) {
			Ok(state) => state,
			Err(PreloadError::Cancelled) => Default::default(),
			e @ Err(_) => e?,
		};

		if !state.complete {
			self.loaded.lock().get_mut(&hash).map(|x| *x &= !(1 << task.preloader.idx));
		}
		if let Some(e) = state.error {
			error!("Error when running preloader '{}':\n{e}", task.preloader.name);
		}

		Ok(self.ops.out(task.id, PreloadOut::Succ))
	}
}

impl Preload {
	pub(crate) fn submit(&self, r#in: PreloadIn) {
		let priority = match r#in.preloader.prio {
			Priority::Low => LOW,
			Priority::Normal => NORMAL,
			Priority::High => HIGH,
		};

		_ = self.tx.try_send(r#in, priority);
	}
}
