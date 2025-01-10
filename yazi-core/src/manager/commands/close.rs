use yazi_shared::event::CmdCow;

use crate::{manager::Manager, tasks::Tasks, manager::commands::quit};

#[derive(Default)]
struct Opt {
	no_cwd_file: bool,
}

impl From<()> for Opt {
	fn from(_: ()) -> Self { Self::default() }
}

impl From<CmdCow> for Opt {
	fn from(c: CmdCow) -> Self { Self { no_cwd_file: c.bool("no-cwd-file") } }
}

impl Manager {
	#[yazi_codegen::command]
	pub fn close(&mut self, opt: Opt, tasks: &Tasks) {
		if self.tabs.len() > 1 {
			return self.tabs.close(self.tabs.cursor);
		}
		self.quit(quit::Opt{ no_cwd_file: opt.no_cwd_file }, tasks);
	}
}
