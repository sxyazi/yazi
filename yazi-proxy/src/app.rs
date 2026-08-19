use yazi_core::app::{PluginOpt, QuitOpt};
use yazi_macro::{emit, relay};
use yazi_shared::id::Id;

pub struct AppProxy;

impl AppProxy {
	pub fn passthrough(id: Id) {
		emit!(Call(relay!(app:passthrough).with("id", id)));
	}

	pub fn plugin_do(opt: PluginOpt) {
		emit!(Call(relay!(app:plugin_do).with_any("opt", opt)));
	}

	pub fn quit(opt: QuitOpt) {
		emit!(Call(relay!(app:quit).with_any("opt", opt)));
	}
}
