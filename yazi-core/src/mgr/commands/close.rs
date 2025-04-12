use yazi_shared::event::CmdCow;

use crate::{mgr::{Mgr, commands::quit}, tasks::Tasks};

#[derive(Default)]
struct Opt(quit::Opt);

impl From<CmdCow> for Opt {
	fn from(c: CmdCow) -> Self { Self(c.into()) }
}

impl Mgr {
	#[yazi_codegen::command]
	pub fn close(&mut self, opt: Opt, tasks: &Tasks) {
		if self.tabs.len() > 1 {
			return self.tabs.close(self.tabs.cursor);
		}
		self.quit(opt.0, tasks);
	}
}
