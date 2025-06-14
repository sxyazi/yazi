use yazi_proxy::options::PluginOpt;
use yazi_shared::Id;

#[derive(Debug)]
pub enum PluginIn {
	Entry(PluginInEntry),
}

impl PluginIn {
	pub fn id(&self) -> Id {
		match self {
			Self::Entry(r#in) => r#in.id,
		}
	}
}

#[derive(Debug)]
pub struct PluginInEntry {
	pub id:  Id,
	pub opt: PluginOpt,
}
