use std::path::MAIN_SEPARATOR;

use crossterm::{execute, terminal::SetTitle};
use yazi_config::YAZI;
use yazi_fs::CWD;
use yazi_shared::{event::CmdCow, tty::TTY};

use crate::{mgr::Mgr, tasks::Tasks};

impl Mgr {
	pub fn refresh(&mut self, _: CmdCow, tasks: &Tasks) {
		if CWD.set(self.cwd()) && !YAZI.mgr.title_format.is_empty() {
			execute!(TTY.writer(), SetTitle(self.title())).ok();
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
		let cwd = if let Ok(p) = self.cwd().strip_prefix(home) {
			format!("~{}{}", MAIN_SEPARATOR, p.display())
		} else {
			format!("{}", self.cwd().display())
		};

		YAZI.mgr.title_format.replace("{cwd}", &cwd)
	}
}
