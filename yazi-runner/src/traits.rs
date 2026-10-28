use yazi_shared::scope::ScopeLike;
use yazi_shim::SStr;

pub trait PluginJob: ScopeLike {
	fn name(&self) -> &SStr;
}
