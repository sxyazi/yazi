use yazi_proxy::options::PluginOpt;

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
	pub id:  usize,
	pub opt: PluginOpt,
}
