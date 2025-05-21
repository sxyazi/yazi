use yazi_proxy::options::PluginOpt;

#[derive(Debug)]
pub enum PluginIn {
	Entry(PluginInEntry),
}

impl PluginIn {
	pub fn id(&self) -> usize {
		match self {
			Self::Entry(r#in) => r#in.id,
		}
	}
}

#[derive(Debug)]
pub struct PluginInEntry {
	pub id:  usize,
	pub opt: PluginOpt,
}
