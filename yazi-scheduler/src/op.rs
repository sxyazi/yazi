use tokio::sync::mpsc;
use yazi_shared::Id;

use crate::TaskOut;

#[derive(Debug)]
pub(crate) struct TaskOp {
	pub(crate) id:  Id,
	pub(crate) out: TaskOut,
}

// --- Ops
#[derive(Clone)]
pub struct TaskOps(pub(super) mpsc::UnboundedSender<TaskOp>);

impl From<&mpsc::UnboundedSender<TaskOp>> for TaskOps {
	fn from(tx: &mpsc::UnboundedSender<TaskOp>) -> Self { Self(tx.clone()) }
}

impl TaskOps {
	#[inline]
	pub(crate) fn out(&self, id: Id, out: impl Into<TaskOut>) {
		_ = self.0.send(TaskOp { id, out: out.into() });
	}
}
