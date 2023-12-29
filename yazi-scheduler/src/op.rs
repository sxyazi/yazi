use crate::{file::FileOp, plugin::PluginOp, preload::PreloadOp};

#[derive(Debug)]
pub enum TaskOp {
	File(Box<FileOp>),
	Plugin(Box<PluginOp>),
	Preload(Box<PreloadOp>),
}

impl TaskOp {
	pub fn id(&self) -> usize {
		match self {
			TaskOp::File(op) => op.id(),
			TaskOp::Plugin(op) => op.id(),
			TaskOp::Preload(op) => op.id(),
		}
	}
}

impl From<FileOp> for TaskOp {
	fn from(op: FileOp) -> Self { Self::File(Box::new(op)) }
}

impl From<PluginOp> for TaskOp {
	fn from(op: PluginOp) -> Self { Self::Plugin(Box::new(op)) }
}

impl From<PreloadOp> for TaskOp {
	fn from(op: PreloadOp) -> Self { Self::Preload(Box::new(op)) }
}
