use yazi_shared::event::CmdCow;

use crate::{mgr::{Mgr, commands::quit}, tasks::Tasks};

#[derive(Default)]
struct Opt {
	no_cwd_file: bool,
}
impl From<CmdCow> for Opt {
	fn from(c: CmdCow) -> Self { Self { no_cwd_file: c.bool("no-cwd-file") } }
}
impl From<Opt> for quit::Opt {
	fn from(value: Opt) -> Self { Self { no_cwd_file: value.no_cwd_file } }
}

impl Mgr {
	#[yazi_codegen::command]
	pub fn close(&mut self, opt: Opt, tasks: &Tasks) {
		if self.tabs.len() > 1 {
			return self.tabs.close(self.tabs.cursor);
		}
		self.quit(opt, tasks);
	}
}
