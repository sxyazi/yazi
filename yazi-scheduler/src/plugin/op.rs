#[derive(Debug)]
pub enum PluginOp {
	Entry(PluginOpEntry),
}

#[derive(Clone, Debug)]
pub struct PluginOpEntry {
	pub id:   usize,
	pub name: String,
}
