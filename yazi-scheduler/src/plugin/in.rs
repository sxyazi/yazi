use yazi_parser::app::PluginOpt;
use yazi_shared::Id;

#[derive(Debug)]
pub(crate) struct PluginInEntry {
	pub(crate) id:  Id,
	pub(crate) opt: PluginOpt,
}
