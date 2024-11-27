use yazi_macro::render;
use yazi_shared::event::CmdCow;

use crate::tab::Tab;

struct Opt {
	prev: bool,
}

impl From<CmdCow> for Opt {
	fn from(c: CmdCow) -> Self { Self { prev: c.bool("previous") } }
}

impl Tab {
	#[yazi_codegen::command]
	pub fn find_arrow(&mut self, opt: Opt) {
		let Some(finder) = &mut self.finder else {
			return;
		};

		render!(finder.catchup(&self.current.files));
		if opt.prev {
			finder.prev(&self.current.files, self.current.cursor, false).map(|s| self.arrow(s));
		} else {
			finder.next(&self.current.files, self.current.cursor, false).map(|s| self.arrow(s));
		}
	}
}
