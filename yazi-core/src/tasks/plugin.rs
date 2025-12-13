use yazi_parser::app::PluginOpt;

use super::Tasks;

impl Tasks {
	pub fn plugin_entry(&self, opt: PluginOpt) { self.scheduler.plugin_entry(opt); }
}
