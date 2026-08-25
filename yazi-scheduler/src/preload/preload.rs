use std::num::NonZeroUsize;

use anyhow::Result;
use lru::LruCache;
use parking_lot::Mutex;
use tokio::sync::mpsc;
use yazi_config::Priority;
use yazi_fs::{FsHash64, file::FileSig};
use yazi_macro::error;
use yazi_runner::{RUNNER, preloader::{PreloadError, PreloadJob}};
use yazi_shared::id::Id;

use crate::{HIGH, LOW, Loaded, NORMAL, TaskOp, TaskOps, preload::{PreloadIn, PreloadInPreload, PreloadOut}};

pub struct Preload {
	ops: TaskOps,
	tx:  async_priority_channel::Sender<PreloadIn, u8>,

	pub loaded:         Mutex<LruCache<u64, Loaded>>,
	pub(crate) loading: Mutex<LruCache<u64, Id>>,
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

	pub(crate) async fn preload(&self, task: PreloadInPreload) -> Result<(), PreloadOut> {
		let hash = FileSig(&task.file).hash_u64();
		let mut rx = RUNNER
			.preload(PreloadJob {
				preloader: task.preloader.clone(),
				file:      task.file,
				mime:      task.mime,
			})
			.await;

		let state = match rx.recv().await.unwrap_or(Err(PreloadError::Cancelled)) {
			Ok(state) => state,
			Err(PreloadError::Cancelled) => Default::default(),
			e @ Err(_) => e?,
		};

		if !state.complete {
			self.loaded.lock().get_mut(&hash).map(|x| x.clear(task.preloader.idx, task.preloader.rev));
		}
		if let Some(e) = state.error {
			error!("Error when running preloader '{}':\n{e}", task.preloader.name);
		}

		Ok(self.ops.out(task.id, PreloadOut::Succ))
	}
}

impl Preload {
	pub(crate) fn submit(&self, r#in: impl Into<PreloadIn>) {
		let r#in = r#in.into();
		let priority = match &r#in {
			PreloadIn::Preload(r#in) => match r#in.preloader.prio {
				Priority::Low => LOW,
				Priority::Normal => NORMAL,
				Priority::High => HIGH,
			},
			PreloadIn::Custom(_) => LOW,
		};

		_ = self.tx.try_send(r#in, priority);
	}
}
