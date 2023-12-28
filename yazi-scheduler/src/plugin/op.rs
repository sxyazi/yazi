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

#[derive(Clone, Debug)]
pub struct PluginOpEntry {
	pub id:   usize,
	pub name: String,
}
