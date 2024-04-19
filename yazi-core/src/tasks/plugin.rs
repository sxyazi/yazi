use yazi_shared::event::Data;

use super::Tasks;

impl Tasks {
	#[inline]
	pub fn plugin_micro(&self, name: String, args: Vec<Data>) {
		self.scheduler.plugin_micro(name, args);
	}

	#[inline]
	pub fn plugin_macro(&self, name: String, args: Vec<Data>) {
		self.scheduler.plugin_macro(name, args);
	}
}
