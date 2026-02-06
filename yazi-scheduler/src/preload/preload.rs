use std::num::NonZeroUsize;

use anyhow::Result;
use lru::LruCache;
use parking_lot::Mutex;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;
use tracing::error;
use yazi_config::Priority;
use yazi_fs::FsHash64;
use yazi_plugin::isolate;

use crate::{HIGH, LOW, NORMAL, TaskOp, TaskOps, preload::{PreloadIn, PreloadOut}};

pub struct Preload {
	ops: TaskOps,
	tx:  async_priority_channel::Sender<PreloadIn, u8>,

	pub loaded:  Mutex<LruCache<u64, u16>>,
	pub loading: Mutex<LruCache<u64, CancellationToken>>,
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
		let ct = CancellationToken::new();
		if let Some(ct) = self.loading.lock().put(task.target.url.hash_u64(), ct.clone()) {
			ct.cancel();
		}

		let hash = task.target.hash_u64();
		let (ok, err) = isolate::preload(&task.plugin.run, task.target, ct).await?;

		if !ok {
			self.loaded.lock().get_mut(&hash).map(|x| *x &= !(1 << task.plugin.idx));
		}
		if let Some(e) = err {
			error!("Error when running preloader `{}`:\n{e}", task.plugin.run.name);
		}

		Ok(self.ops.out(task.id, PreloadOut::Succ))
	}
}

impl Preload {
	pub(crate) fn submit(&self, r#in: PreloadIn) {
		let priority = match r#in.plugin.prio {
			Priority::Low => LOW,
			Priority::Normal => NORMAL,
			Priority::High => HIGH,
		};

		_ = self.tx.try_send(r#in.into(), priority);
	}
}
