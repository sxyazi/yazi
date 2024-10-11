use yazi_shared::{event::Cmd, render};

use crate::tab::Tab;

pub struct Opt {
	prev: bool,
}

impl From<Cmd> for Opt {
	fn from(c: Cmd) -> Self { Self { prev: c.bool("previous") } }
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
