#[derive(Debug)]
pub enum TaskOp {
	File(Box<crate::file::FileOp>),
	Plugin(Box<crate::plugin::PluginOp>),
	Preload(Box<crate::preload::PreloadOp>),
	Process(Box<crate::process::ProcessOp>),
}

impl TaskOp {
	pub fn id(&self) -> usize {
		match self {
			TaskOp::File(op) => op.id(),
			TaskOp::Plugin(op) => op.id(),
			TaskOp::Preload(op) => op.id(),
			TaskOp::Process(op) => op.id(),
		}
	}
}
