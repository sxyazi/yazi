use std::num::NonZeroUsize;

use anyhow::Result;
use hashbrown::{HashMap, HashSet};
use lru::LruCache;
use parking_lot::{Mutex, RwLock};
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;
use tracing::error;
use yazi_config::Priority;
use yazi_fs::{FilesOp, FsHash64};
use yazi_plugin::isolate;
use yazi_shared::{event::CmdCow, url::{UrlBuf, UrlLike}};
use yazi_vfs::provider;

use super::{PreworkInFetch, PreworkInLoad, PreworkInSize};
use crate::{HIGH, NORMAL, TaskIn, TaskOp, TaskOps, prework::{PreworkOutFetch, PreworkOutLoad, PreworkOutSize}};

pub struct Prework {
	ops:     TaskOps,
	r#macro: async_priority_channel::Sender<TaskIn, u8>,

	pub loaded:  Mutex<LruCache<u64, u32>>,
	pub loading: Mutex<LruCache<u64, CancellationToken>>,
	pub sizing:  RwLock<HashSet<UrlBuf>>,
}

impl Prework {
	pub(crate) fn new(
		ops: &mpsc::UnboundedSender<TaskOp>,
		r#macro: &async_priority_channel::Sender<TaskIn, u8>,
	) -> Self {
		Self {
			ops:     ops.into(),
			r#macro: r#macro.clone(),
			loaded:  Mutex::new(LruCache::new(NonZeroUsize::new(4096).unwrap())),
			loading: Mutex::new(LruCache::new(NonZeroUsize::new(256).unwrap())),
			sizing:  Default::default(),
		}
	}

	pub(crate) async fn fetch(&self, task: PreworkInFetch) -> Result<(), PreworkOutFetch> {
		match task.plugin.prio {
			Priority::Low => Ok(self.queue(task, NORMAL)),
			Priority::Normal => Ok(self.queue(task, HIGH)),
			Priority::High => self.fetch_do(task).await,
		}
	}

	pub(crate) async fn fetch_do(&self, task: PreworkInFetch) -> Result<(), PreworkOutFetch> {
		let hashes: Vec<_> = task.targets.iter().map(|f| f.hash_u64()).collect();
		let (state, err) = isolate::fetch(CmdCow::from(&task.plugin.run), task.targets).await?;

		let mut loaded = self.loaded.lock();
		for (_, h) in hashes.into_iter().enumerate().filter(|&(i, _)| !state.get(i)) {
			loaded.get_mut(&h).map(|x| *x &= !(1 << task.plugin.idx));
		}
		if let Some(e) = err {
			error!("Error when running fetcher `{}`:\n{e}", task.plugin.run.name);
		}

		Ok(self.ops.out(task.id, PreworkOutFetch::Succ))
	}

	pub(crate) async fn load(&self, task: PreworkInLoad) -> Result<(), PreworkOutLoad> {
		match task.plugin.prio {
			Priority::Low => Ok(self.queue(task, NORMAL)),
			Priority::Normal => Ok(self.queue(task, HIGH)),
			Priority::High => self.load_do(task).await,
		}
	}

	pub(crate) async fn load_do(&self, task: PreworkInLoad) -> Result<(), PreworkOutLoad> {
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

		Ok(self.ops.out(task.id, PreworkOutLoad::Succ))
	}

	pub(crate) async fn size(&self, task: PreworkInSize) -> Result<(), PreworkOutSize> {
		self.size_do(task).await
	}

	pub(crate) async fn size_do(&self, task: PreworkInSize) -> Result<(), PreworkOutSize> {
		let length = provider::calculate(&task.target).await.unwrap_or(0);
		task.throttle.done((task.target, length), |buf| {
			{
				let mut loading = self.sizing.write();
				for (path, _) in &buf {
					loading.remove(path);
				}
			}

			let parent = buf[0].0.parent().unwrap();
			FilesOp::Size(
				parent.into(),
				HashMap::from_iter(buf.into_iter().map(|(u, s)| (u.urn().into(), s))),
			)
			.emit();
		});

		Ok(self.ops.out(task.id, PreworkOutSize::Done))
	}
}

impl Prework {
	#[inline]
	fn queue(&self, r#in: impl Into<TaskIn>, priority: u8) {
		_ = self.r#macro.try_send(r#in.into(), priority);
	}
}
