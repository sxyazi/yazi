use yazi_shared::event::Data;

#[derive(Debug)]
pub enum PluginOp {
	Entry(PluginOpEntry),
}

impl PluginOp {
	pub fn id(&self) -> usize {
		match self {
			Self::Entry(op) => op.id,
		}
	}
}

#[derive(Debug)]
pub struct PluginOpEntry {
	pub id:   usize,
	// TODO: remove these fields and use `CmdCow` instead
	pub name: String,
	pub args: Vec<Data>,
}
