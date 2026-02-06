use yazi_parser::app::PluginOpt;
use yazi_shared::Id;

#[derive(Debug)]
pub(crate) enum PluginIn {
	Entry(PluginInEntry),
}

impl_from_in!(Entry(PluginInEntry));

impl PluginIn {
	pub(crate) fn id(&self) -> Id {
		match self {
			Self::Entry(r#in) => r#in.id,
		}
	}
}

// --- Entry
#[derive(Debug)]
pub(crate) struct PluginInEntry {
	pub(crate) id:  Id,
	pub(crate) opt: PluginOpt,
}
