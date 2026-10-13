use yazi_binding::Scope;
use yazi_shared::id::Id;
use yazi_shim::SStr;

pub trait PluginJob {
	fn tab(&self) -> Id;

	fn name(&self) -> &SStr;

	/// The cancellation scope the plugin runs under. Jobs that are not attached
	/// to a cancelable task return `None` here, which makes the plugin run
	/// under a detached scope.
	fn scope(&self) -> Option<Scope> { None }
}
