use yazi_shared::event::{CmdCow, Data};

use crate::{mgr::{Mgr, commands::quit}, tasks::Tasks};

#[derive(Default)]
struct Opt {
	no_cwd_file: bool,
	exit_code:   i32,
}
impl From<CmdCow> for Opt {
	fn from(c: CmdCow) -> Self {
		Self {
			no_cwd_file: c.bool("no-cwd-file"),
			exit_code:   c.get("exit_code").and_then(Data::as_i32).unwrap_or_default(),
		}
	}
}
impl From<Opt> for quit::Opt {
	fn from(value: Opt) -> Self {
		Self { no_cwd_file: value.no_cwd_file, exit_code: value.exit_code }
	}
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
