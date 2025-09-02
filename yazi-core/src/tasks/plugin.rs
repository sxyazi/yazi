use yazi_parser::app::PluginOpt;

use super::Tasks;

impl Tasks {
	pub fn plugin_micro(&self, opt: PluginOpt) { self.scheduler.plugin_micro(opt); }

	pub fn plugin_macro(&self, opt: PluginOpt) { self.scheduler.plugin_macro(opt); }
}
