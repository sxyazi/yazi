use yazi_shared::Id;

use crate::{file::FileIn, plugin::PluginIn, prework::PreworkIn};

#[derive(Debug)]
pub enum TaskOp {
	File(Box<FileIn>),
	Plugin(Box<PluginIn>),
	Prework(Box<PreworkIn>),
}

impl TaskOp {
	pub fn id(&self) -> Id {
		match self {
			TaskOp::File(r#in) => r#in.id(),
			TaskOp::Plugin(r#in) => r#in.id(),
			TaskOp::Prework(r#in) => r#in.id(),
		}
	}
}

impl From<FileIn> for TaskOp {
	fn from(r#in: FileIn) -> Self { Self::File(Box::new(r#in)) }
}

impl From<PluginIn> for TaskOp {
	fn from(r#in: PluginIn) -> Self { Self::Plugin(Box::new(r#in)) }
}

impl From<PreworkIn> for TaskOp {
	fn from(r#in: PreworkIn) -> Self { Self::Prework(Box::new(r#in)) }
}
