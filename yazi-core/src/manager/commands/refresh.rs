use std::{env, path::MAIN_SEPARATOR};

use crossterm::{execute, terminal::SetTitle};
use yazi_config::MANAGER;
use yazi_shared::event::Cmd;

use crate::{manager::Manager, tasks::Tasks};

impl Manager {
	pub fn refresh(&mut self, _: Cmd, tasks: &Tasks) {
		env::set_current_dir(self.cwd()).ok();
		env::set_var("PWD", self.cwd());

		if !MANAGER.title_format.is_empty() {
			execute!(std::io::stderr(), SetTitle(self.title())).ok();
		}

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

	fn title(&self) -> String {
		let home = dirs::home_dir().unwrap_or_default();
		let cwd = if let Some(p) = self.cwd().strip_prefix(home) {
			format!("~{}{}", MAIN_SEPARATOR, p.display())
		} else {
			format!("{}", self.cwd().display())
		};

		MANAGER.title_format.replace("{cwd}", &cwd)
	}
}
