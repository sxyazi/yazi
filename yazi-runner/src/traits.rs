use yazi_shared::id::Id;
use yazi_shim::SStr;

pub trait PluginJob {
	fn tab(&self) -> Id;

	fn name(&self) -> &SStr;
}
