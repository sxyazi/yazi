use std::env;

use crossterm::{execute, terminal::SetTitle};
use yazi_shared::event::Cmd;

use crate::{manager::Manager, tasks::Tasks};

impl Manager {
	fn title(&self) -> String {
		#[cfg(unix)]
		let home = env::var_os("HOME").unwrap_or_default();
		#[cfg(windows)]
		let home = env::var_os("USERPROFILE").unwrap_or_default();

		if let Some(p) = self.cwd().strip_prefix(home) {
			format!("Yazi: ~/{}", p.display())
		} else {
			format!("Yazi: {}", self.cwd().display())
		}
	}

	pub fn refresh(&mut self, _: Cmd, tasks: &Tasks) {
		env::set_current_dir(self.cwd()).ok();
		env::set_var("PWD", self.cwd());
		execute!(std::io::stderr(), SetTitle(self.title())).ok();

		self.active_mut().apply_files_attrs();

		if let Some(p) = self.parent() {
			self.watcher.trigger_dirs(&[self.current(), p]);
		} else {
			self.watcher.trigger_dirs(&[self.current()]);
		}

		self.hover(None);
		self.update_paged((), tasks);

		tasks.prework_sorted(&self.current().files);
	}
}
