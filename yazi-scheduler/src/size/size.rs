use anyhow::Result;
use hashbrown::{HashMap, HashSet};
use parking_lot::RwLock;
use tokio::sync::mpsc;
use yazi_fs::FilesOp;
use yazi_shared::url::{UrlBuf, UrlLike};
use yazi_vfs::provider;

use super::SizeIn;
use crate::{TaskOp, TaskOps, size::SizeOut};

pub struct Size {
	ops: TaskOps,
	tx:  async_priority_channel::Sender<SizeIn, u8>,

	pub sizing: RwLock<HashSet<UrlBuf>>,
}

impl Size {
	pub(crate) fn new(
		ops: &mpsc::UnboundedSender<TaskOp>,
		tx: async_priority_channel::Sender<SizeIn, u8>,
	) -> Self {
		Self { ops: ops.into(), tx, sizing: Default::default() }
	}

	pub(crate) async fn size(&self, task: SizeIn) -> Result<(), SizeOut> {
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

		Ok(self.ops.out(task.id, SizeOut::Done))
	}
}

impl Size {
	#[inline]
	pub(crate) fn submit(&self, r#in: impl Into<SizeIn>, priority: u8) {
		_ = self.tx.try_send(r#in.into(), priority);
	}
}
