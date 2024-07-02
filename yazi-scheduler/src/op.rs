use crate::{file::FileOp, plugin::PluginOp, prework::PreworkOp};

#[derive(Debug)]
pub enum TaskOp {
	File(Box<FileOp>),
	Plugin(Box<PluginOp>),
	Prework(Box<PreworkOp>),
}

impl TaskOp {
	pub fn id(&self) -> usize {
		match self {
			TaskOp::File(op) => op.id(),
			TaskOp::Plugin(op) => op.id(),
			TaskOp::Prework(op) => op.id(),
		}
	}
}

impl From<FileOp> for TaskOp {
	fn from(op: FileOp) -> Self { Self::File(Box::new(op)) }
}

impl From<PluginOp> for TaskOp {
	fn from(op: PluginOp) -> Self { Self::Plugin(Box::new(op)) }
}

impl From<PreworkOp> for TaskOp {
	fn from(op: PreworkOp) -> Self { Self::Prework(Box::new(op)) }
}
