use yazi_proxy::options::PluginOpt;

use super::Tasks;

impl Tasks {
	#[inline]
	pub fn plugin_micro(&self, opt: PluginOpt) { self.scheduler.plugin_micro(opt); }

	#[inline]
	pub fn plugin_macro(&self, opt: PluginOpt) { self.scheduler.plugin_macro(opt); }
}
