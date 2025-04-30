use crossterm::{execute, terminal::SetTitle};
use yazi_config::YAZI;
use yazi_fs::CWD;
use yazi_shared::event::CmdCow;
use yazi_term::tty::TTY;

use crate::{mgr::Mgr, tasks::Tasks};

impl Mgr {
	pub fn refresh(&mut self, _: CmdCow, tasks: &Tasks) {
		if let (_, Some(s)) = (CWD.set(self.cwd()), YAZI.mgr.title()) {
			execute!(TTY.writer(), SetTitle(s)).ok();
		}

		self.active_mut().apply_files_attrs();

		if let Some(p) = self.parent() {
			self.watcher.trigger_dirs(&[self.current(), p]);
		} else {
			self.watcher.trigger_dirs(&[self.current()]);
		}

		self.peek(false);
		self.watch(());
		self.update_paged((), tasks);

		tasks.prework_sorted(&self.current().files);
	}
}
