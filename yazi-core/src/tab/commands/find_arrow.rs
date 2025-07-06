use yazi_macro::render;
use yazi_parser::tab::FindArrowOpt;

use crate::tab::Tab;

impl Tab {
	#[yazi_codegen::command]
	pub fn find_arrow(&mut self, opt: FindArrowOpt) {
		let Some(finder) = &mut self.finder else {
			return;
		};

		render!(finder.catchup(&self.current));
		if opt.prev {
			finder.prev(&self.current.files, self.current.cursor, false).map(|s| self.arrow(s));
		} else {
			finder.next(&self.current.files, self.current.cursor, false).map(|s| self.arrow(s));
		}
	}
}
