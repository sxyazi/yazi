use std::env;

use yazi_shared::{emit, event::Exec, Layer};

use crate::manager::Manager;

impl Manager {
	#[inline]
	pub fn _refresh() {
		emit!(Call(Exec::call("refresh", vec![]).vec(), Layer::Manager));
	}

	pub fn refresh(&mut self, _: &Exec) -> bool {
		env::set_current_dir(self.cwd()).ok();
		env::set_var("PWD", self.cwd());

		self.active_mut().apply_files_attrs(false);

		if let Some(p) = self.parent() {
			self.watcher.trigger_dirs(&[self.cwd(), &p.cwd]);
		} else {
			self.watcher.trigger_dirs(&[self.cwd()]);
		}

		self.hover(None)
	}
}
