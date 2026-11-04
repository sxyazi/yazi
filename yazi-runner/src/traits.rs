use yazi_shared::scope::ScopeLike;
use yazi_shim::CompactString;

pub trait PluginJob: ScopeLike {
	fn name(&self) -> &CompactString;
}
