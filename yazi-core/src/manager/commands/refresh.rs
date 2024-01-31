use std::env;

use yazi_shared::{emit, event::Cmd, Layer};

use crate::{manager::Manager, tasks::Tasks};

impl Manager {
	#[inline]
	pub fn _refresh() {
		emit!(Call(Cmd::new("refresh"), Layer::Manager));
	}

	pub fn refresh(&mut self, _: Cmd, tasks: &Tasks) {
		env::set_current_dir(self.cwd()).ok();
		env::set_var("PWD", self.cwd());

		self.active_mut().apply_files_attrs();

		if let Some(p) = self.parent() {
			self.watcher.trigger_dirs(&[self.current(), p]);
		} else {
			self.watcher.trigger_dirs(&[self.current()]);
		}

		self.hover(None);
		self.update_paged((), tasks);

		tasks.preload_sorted(&self.current().files);
	}
}
